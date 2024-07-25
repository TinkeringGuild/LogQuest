use lazy_static::lazy_static;
use regex::Regex;
use std::{
  collections::{HashMap, LinkedList},
  ops::Index,
};

use crate::utils::random_id;

lazy_static! {
  static ref NO_VALUE: String = String::new();

  // This enables regular expressions to support the weird GINA-style variables
  static ref REGEX_VARS: Regex =
    Regex::new(r"\{\s*(?:([Cc]|[Ss]\d*)|(?:([Nn]\d*)\s*(?:(>=|<=|=|>|<)\s*(-?\d+))?))\s*\}").unwrap();

  static ref GENERATED_NAMED_CAPTURE_NAME: Regex = Regex::new(r"^LQ[A-Z0-9]{8}$").unwrap();
}

struct RegexGINA {
  compiled: Regex,
  named_projections: HashMap<String, String>,
  positional_projections: Vec<usize>,
  conditions: LinkedList<Box<dyn Fn(&regex::Captures) -> bool>>,
}

impl RegexGINA {
  pub fn from_str(pattern: &str) -> anyhow::Result<Self> {
    // A lot of the complexity here comes from how special GINA tokens
    // like {S} or {N>=10} are extracted using named capture groups, but
    // the inclusion of these capture groups must be invisible to the
    // end-user writing a Filter; notably, they should be able to address
    // their regex's capture groups by index without these dynamically
    // interpolated capture groups affecting the indices they'd expect.

    let mut named_projections: HashMap<String, String> = HashMap::new();

    let mut conditions: LinkedList<Box<dyn Fn(&regex::Captures) -> bool>> = LinkedList::new();
    let with_replacements = REGEX_VARS.replace_all(pattern, |captures: &regex::Captures| {
      let projected_from = Self::generate_named_capture_name();

      if let (Some(projected_name), Some(operator), Some(operand)) =
        (captures.get(2), captures.get(3), captures.get(4))
      {
        let operator = operator.as_str().to_owned();
        let operand: i64 = operand
          .as_str()
          .parse()
          .expect("regex is supposed to guarantee numeric type!");
        let projected_to = projected_name.as_str().to_uppercase();
        let projected_from_ = projected_from.clone();
        named_projections.insert(projected_from.clone(), projected_to.clone());
        let condition = move |caps: &regex::Captures| -> bool {
          if let Some(value) = caps.name(&projected_from_) {
            let value: i64 = value
              .as_str()
              .parse()
              .expect("regex should be validating this is numeric!");
            match operator.as_str() {
              "=" => value == operand,
              "<=" => value <= operand,
              ">=" => value >= operand,
              ">" => value > operand,
              "<" => value < operand,
              _ => unreachable!(/* REGEX_VARS only allows the operators above */),
            }
          } else {
            true
          }
        };
        conditions.push_back(Box::new(condition));
        Self::pattern_for_number_capture(&projected_from)
      } else if let Some(projected_to) = captures.get(2) {
        // if here, we have an N-case without constraints, e.g. {N} or {N2}
        let projected_to = projected_to.as_str().to_uppercase();
        named_projections.insert(projected_from.clone(), projected_to);
        Self::pattern_for_number_capture(&projected_from)
      } else if let Some(projected_to) = captures.get(1) {
        let projected_to = projected_to.as_str().to_uppercase();
        named_projections.insert(projected_from.clone(), projected_to.clone());
        if projected_to == "C" {
          Self::pattern_for_character_name_capture(&projected_from)
        } else {
          // if here, we have a S-case, e.g. {S} or {S1}
          Self::pattern_for_string_capture(&projected_from)
        }
      } else {
        unreachable!(/* For REGEX_VARS to match, there will always be a capture */)
      }
    });

    let compiled = Regex::new(&with_replacements)?;

    // Look up an index to get the index from the Captures that corresponds to the index
    let mut positional_projections = vec![0];
    for (index, cap) in compiled.capture_names().enumerate() {
      if index == 0 {
        continue;
      }
      if let Some(capture_name) = cap {
        if GENERATED_NAMED_CAPTURE_NAME.is_match(capture_name) {
          continue;
        }
        named_projections.insert(capture_name.to_owned(), capture_name.to_owned());
      }
      positional_projections.push(index);
    }

    Ok(Self {
      compiled,
      named_projections,
      positional_projections,
      conditions,
    })
  }

  pub fn test(&self, haystack: &str, character_name: &str) -> Option<CapturesGINA> {
    let direct_captures: regex::Captures = match self.compiled.captures(haystack) {
      Some(captures) => captures,
      None => return None,
    };

    for condition in self.conditions.iter() {
      if !condition(&direct_captures) {
        return None;
      }
    }

    let mut named_captures = HashMap::<String, String>::new();
    for (capture_name, output_name) in self.named_projections.iter() {
      let Some(captured_value) = direct_captures
        .name(&capture_name)
        .and_then(|m| Some(m.as_str().to_owned()))
      else {
        continue;
      };
      if let Some(replaced_value) =
        named_captures.insert(output_name.to_uppercase(), captured_value.clone())
      {
        // Make sure all values for a given named capture are equal
        if replaced_value != captured_value {
          return None;
        }
      }
    }

    if let Some(captured_character_name) = named_captures.get("C") {
      if captured_character_name != character_name {
        return None;
      }
    }

    let positional_captures: Vec<Option<String>> = self
      .positional_projections
      .iter()
      .map(|i| direct_captures.get(*i).map(|m| m.as_str().to_owned()))
      .collect();

    Some(CapturesGINA {
      positional_captures,
      named_captures,
      character_name: character_name.to_owned(),
    })
  }

  fn generate_named_capture_name() -> String {
    format!("LQ{}", random_id(8)) // This must be consistent with GENERATED_NAMED_CAPTURE_NAME
  }

  fn group_values_by_key(hashmap: &HashMap<String, String>) -> HashMap<String, Vec<String>> {
    let mut returned: HashMap<String, Vec<String>> = HashMap::new();
    for (key, value) in hashmap {
      returned
        .entry(value.to_owned())
        .or_insert_with(Vec::new)
        .push(key.to_owned());
    }
    returned
  }

  fn pattern_for_number_capture(capture_name: &str) -> String {
    format!(r"(?<{capture_name}>-?\d+)")
  }

  fn pattern_for_string_capture(capture_name: &str) -> String {
    // This pattern MIGHT also need to capture underscores if a good example can be found for it.
    format!(r"(?<{capture_name}>[\w'`-](?:[ \w'`-]*[\w'`-]+)?)") // ensures {S} never ends in a space
  }

  fn pattern_for_character_name_capture(capture_name: &str) -> String {
    format!(r"(?<{}>{})", capture_name, r"[A-Za-z]{3,15}") // On P99, 3-letter toon names do exist
  }
}

struct CapturesGINA {
  character_name: String,
  positional_captures: Vec<Option<String>>,
  named_captures: HashMap<String, String>,
}

impl Index<usize> for CapturesGINA {
  type Output = String;

  fn index(&self, index: usize) -> &Self::Output {
    if index >= self.positional_captures.len() {
      return &NO_VALUE;
    }
    self.positional_captures[index]
      .as_ref()
      .unwrap_or(&NO_VALUE)
  }
}

impl Index<&str> for CapturesGINA {
  type Output = String;

  fn index(&self, key: &str) -> &Self::Output {
    if let Ok(numeric_key) = key.parse::<usize>() {
      return &self[numeric_key];
    }
    let key = key.to_uppercase();
    if key == "C" {
      return &self.character_name;
    }
    if let Some(value) = self.named_captures.get(&key) {
      value
    } else {
      &NO_VALUE
    }
  }
}

impl Index<String> for CapturesGINA {
  type Output = String;

  fn index(&self, key: String) -> &Self::Output {
    &self[key.as_str()]
  }
}

// TODO {S} should not be allowed to end in a space. What string lets me test that case?
#[cfg(test)]
mod tests {
  use super::*;

  const TEST_CHARACTER_NAME: &str = "Xenk";

  #[test]
  fn test_multiple_string_matches() {
    assert_pattern_matches(
      r"^{S} hits {S1}$",
      "Yelinak hits King Tormax",
      &[("S", "Yelinak"), ("S1", "King Tormax")],
    );
    assert_pattern_matches(
      r"^{S1} hits {S2}$",
      "Yelinak hits King Tormax",
      &[("S1", "Yelinak"), ("S2", "King Tormax")],
    );
    assert_pattern_matches(
      r"^{S1} hits {S2} but {S2} ripostes!$",
      "Yelinak hits King Tormax but King Tormax ripostes!",
      &[("S1", "Yelinak"), ("S2", "King Tormax")],
    );
    assert_pattern_does_not_match(r"^{S} hits {S}$", &["Yelinak hits King Tormax"]);
  }

  #[test]
  fn test_branches_with_optional_captures() {
    assert_pattern_matches(
      r"^You (must target an NPC to taunt first\.|taunt {S} to ignore others and attack you\!)$",
      "You must target an NPC to taunt first.",
      &[("S", "")],
    );
    assert_pattern_matches(
      r"^You (must target an NPC to taunt first\.|taunt {S} to ignore others and attack you\!)$",
      "You taunt Bristlebane to ignore others and attack you!",
      &[("S", "Bristlebane")],
    );
  }

  #[test]
  fn test_referencing_invalid_captures() {
    assert_pattern_matches(
      r"^This has no captures$",
      "This has no captures",
      &[("S", ""), ("S9999", ""), ("N", ""), ("N100", "")],
    );
  }

  #[test]
  fn test_referencing_character_name() {
    assert_pattern_matches(
      r".+",
      "This is a test",
      &[("C", TEST_CHARACTER_NAME), ("c", TEST_CHARACTER_NAME)],
    );
    assert_pattern_matches(
      r"^Hail, {C}$",
      &format!("Hail, {TEST_CHARACTER_NAME}"),
      &[("C", TEST_CHARACTER_NAME), ("c", TEST_CHARACTER_NAME)],
    );
    assert_pattern_does_not_match(r"^Hail, {C}$", &["Hail, Incorrect"]);
    assert_pattern_does_not_match(
      r"^You, {C}, are named {C}$",
      &[&format!("You, {TEST_CHARACTER_NAME}, are named Incorrect")],
    )
  }

  #[test]
  fn test_numeric_type_doesnt_match_characters() {
    assert_pattern_does_not_match(
      r"^You hit {S} for {N} points of damage",
      &["You hit Xenk for ShouldNotMatch points of damage"],
    );
  }

  #[test]
  fn test_numeric_token_with_constraint() {
    // > operator
    {
      assert_pattern_matches(
        r"^You have healed {S} for {N>5750} points? of damage\.$",
        "You have healed Xenk for 6000 points of damage.",
        &[("N", "6000"), ("S", "Xenk")],
      );
      assert_pattern_does_not_match(
        r"^You have healed {S} for {N>5750} points? of damage\.$",
        &[
          "You have healed Xenk for 5750 points of damage.",
          "You have healed Xenk for 1 point of damage.",
        ],
      );
    }

    // < operator
    {
      assert_pattern_matches(
        r"^You have healed {S} for { N < 6000 } points? of damage\.$",
        "You have healed Xenk for 1000 points of damage.",
        &[("N", "1000"), ("S", "Xenk")],
      );
      assert_pattern_does_not_match(
        r"^You have healed {S} for {N<6000} points? of damage\.$",
        &[
          "You have healed Xenk for 6000 points of damage.",
          "You have healed Xenk for 6001 point of damage.",
        ],
      );
    }

    // <= operator
    {
      assert_pattern_matches(
        r"^You have healed {S} for { N <= 6000 } points? of damage\.$",
        "You have healed Xenk for 100 points of damage.",
        &[("N", "100"), ("S", "Xenk")],
      );
      assert_pattern_matches(
        r"^You have healed {S} for { N <= 6000 } points? of damage\.$",
        "You have healed Xenk for 6000 points of damage.",
        &[("N", "6000"), ("S", "Xenk")],
      );
      assert_pattern_does_not_match(
        r"^You have healed {S} for {N<6000} points? of damage\.$",
        &[
          "You have healed Xenk for 6000 points of damage.",
          "You have healed Xenk for 6001 points of damage.",
        ],
      );
    }

    // >= operator
    {
      assert_pattern_matches(
        r"^You have healed {S} for { N >= 6000 } points? of damage\.$",
        "You have healed Xenk for 6000 points of damage.",
        &[("N", "6000"), ("S", "Xenk")],
      );
      assert_pattern_matches(
        r"^You have healed {S} for { N >= 6000 } points? of damage\.$",
        "You have healed Xenk for 6001 points of damage.",
        &[("N", "6001"), ("S", "Xenk")],
      );
      assert_pattern_does_not_match(
        r"^You have healed {S} for {N>=6000} points? of damage\.$",
        &[
          "You have healed Xenk for 5999 points of damage.",
          "You have healed Xenk for 1 point of damage.",
        ],
      );
    }

    // = operator
    {
      assert_pattern_matches(
        r"^You have healed {S} for { N = 6000 } points? of damage\.$",
        "You have healed Xenk for 6000 points of damage.",
        &[("N", "6000"), ("S", "Xenk")],
      );
      assert_pattern_does_not_match(
        r"^You have healed {S} for {N=6000} points? of damage\.$",
        &[
          "You have healed Xenk for 5999 points of damage.",
          "You have healed Xenk for 6001 points of damage.",
          "You have healed Xenk for 1 point of damage.",
        ],
      );
    }
  }

  #[test]
  fn test_accessing_captures_by_index_as_string() {
    // TODO: a numeric capture should be addressable as a string or a usize type.
    assert_pattern_matches(
      r"^Here're some words: (\w+), (\w+), (?<conjunction>\w+) (\w+)$",
      "Here're some words: one, two, and three",
      &[
        ("1", "one"),
        ("2", "two"),
        ("4", "three"),
        ("conjunction", "and"),
        ("CONJUNCTION", "and"),
      ],
    )
  }

  #[test]
  fn test_optional_positional_captures() {
    assert_pattern_matches(
      r"^There are (?:(\w+) parameters (\w+)|none) here$",
      "There are none here",
      &[("1", ""), ("2", "")],
    );
    assert_pattern_matches(
      r"^There are (?:(\w+) parameters (\w+)|none) here$",
      "There are two parameters right here",
      &[("1", "two"), ("2", "right")],
    );
  }

  #[test]
  fn test_accessing_captures_by_index_as_numeric_type() {
    let pattern = r"^Here're some words: (\w+), (\w+), (?<conjunction>\w+) (\w+)$";
    let text = "Here're some words: one, two, and three";

    let rg = RegexGINA::from_str(pattern).expect("Invalid regex pattern");
    let result = rg
      .test(text, TEST_CHARACTER_NAME)
      .expect("RegexGINA did not match!");

    assert_eq!(result[1], "one");
    assert_eq!(result[2], "two");
    assert_eq!(result[3], "and");
    assert_eq!(result[4], "three");
    assert_eq!(result["conjunction"], "and");
    assert_eq!(result["CONJUNCTION"], "and");
  }
  fn assert_pattern_does_not_match(pattern: &str, texts: &[&str]) {
    let rg = RegexGINA::from_str(pattern).expect("Invalid regex pattern");
    for text in texts {
      assert!(rg.test(*text, TEST_CHARACTER_NAME).is_none());
    }
  }

  fn assert_pattern_matches(pattern: &str, text: &str, expectations: &[(&str, &str)]) {
    let rg = RegexGINA::from_str(pattern).expect("Invalid regex pattern");
    let result = rg
      .test(text, TEST_CHARACTER_NAME)
      .expect("RegexGINA did not match!");
    for (key, value) in expectations {
      assert_eq!(result[*key], *value);
    }
  }
}
