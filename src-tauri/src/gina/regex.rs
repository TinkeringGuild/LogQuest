use crate::common::random_id;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, LinkedList};
use std::ops::Index;

lazy_static::lazy_static! {
  static ref NO_VALUE: String = String::new();

  /// For extracting out GINA variable placeholders from a GINA regex (e.g. {S}, {N>100}, {C}, {S2})
  static ref REGEX_VARS: Regex =
    Regex::new(r"\{\s*(?:([Cc]|[Ss]\d*)|(?:([Nn]\d*)\s*(?:(>=|<=|=|>|<)\s*(-?\d+))?))\s*\}").unwrap();

  /// Named capture groups are generated and injected into the converted Regex; this matches the generated names
  static ref GENERATED_NAMED_CAPTURE_NAME: Regex = Regex::new(r"^LQ[A-Z0-9]{8}$").unwrap();
}

type ConditionsList = LinkedList<Box<dyn Fn(&regex::Captures) -> bool + Send + Sync + 'static>>;
struct Conditions(ConditionsList);

#[derive(Debug)]
pub struct RegexGINA {
  raw: String,
  compiled: Regex,
  named_projections: HashMap<String, String>,
  positional_projections: Vec<usize>,
  conditions: Conditions,
}

pub struct CapturesGINA {
  character_name: String,
  positional_captures: Vec<Option<String>>,
  named_captures: HashMap<String, String>,
}

impl RegexGINA {
  // A lot of the complexity here comes from how special GINA tokens
  // like {S} or {N>=10} are extracted using named capture groups, but
  // the inclusion of these capture groups must be invisible to the
  // end-user writing a Filter; notably, they should be able to address
  // their regex's capture groups by index without these dynamically
  // interpolated capture groups affecting the indices they'd expect.
  pub fn from_str(pattern: &str) -> anyhow::Result<Self> {
    let mut named_projections: HashMap<String, String> = HashMap::new();

    let mut conditions: ConditionsList = LinkedList::new();
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
        named_projections.insert(projected_from.clone(), projected_to.clone());
        let condition =
          Self::create_condition_for_numeric_constraints(operator, operand, projected_from.clone());
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
      raw: pattern.to_owned(),
      compiled,
      named_projections,
      positional_projections,
      conditions: Conditions(conditions),
    })
  }

  pub fn check(&self, haystack: &str, character_name: &str) -> Option<CapturesGINA> {
    let direct_captures: regex::Captures = match self.compiled.captures(haystack) {
      Some(captures) => captures,
      None => return None,
    };

    for condition in self.conditions.0.iter() {
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

  // This must be consistent with GENERATED_NAMED_CAPTURE_NAME
  fn generate_named_capture_name() -> String {
    format!("LQ{}", random_id(8)) // 8 makes chance of two collisions approx 1/2.8e12
  }

  fn pattern_for_number_capture(capture_name: &str) -> String {
    format!(r"(?<{capture_name}>-?\d+)")
  }

  fn pattern_for_string_capture(capture_name: &str) -> String {
    // TODO: This pattern MIGHT also need to capture underscores if a good example can be found for it.
    format!(r"(?<{capture_name}>[\w'`-](?:[ \w'`-]*[\w'`-])?)") // ensures {S} never ends in a space
  }

  fn pattern_for_character_name_capture(capture_name: &str) -> String {
    format!(r"(?<{}>{})", capture_name, r"[A-Za-z]{3,15}") // On P99, 3-letter toon names do exist
  }

  fn create_condition_for_numeric_constraints(
    operator: String,
    operand: i64,
    projected_from: String,
  ) -> impl Fn(&regex::Captures) -> bool + Send + 'static {
    move |caps: &regex::Captures| {
      Self::check_numeric_constraints(operator.clone(), operand, projected_from.clone(), caps)
    }
  }

  fn check_numeric_constraints(
    operator: String,
    operand: i64,
    projected_from: String,
    caps: &regex::Captures,
  ) -> bool {
    if let Some(value) = caps.name(&projected_from) {
      let value: i64 = value
        .as_str()
        .parse()
        .expect("regex should be validating this is numeric!");
      return match operator.as_str() {
        "=" => value == operand,
        "<=" => value <= operand,
        ">=" => value >= operand,
        ">" => value > operand,
        "<" => value < operand,
        _ => unreachable!(/* REGEX_VARS only allows the operators above */),
      };
    }
    true
  }
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
    self.named_captures.get(&key).unwrap_or(&NO_VALUE)
  }
}

impl Index<String> for CapturesGINA {
  type Output = String;

  fn index(&self, key: String) -> &Self::Output {
    &self[key.as_str()]
  }
}

impl Serialize for RegexGINA {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.raw.as_str())
  }
}

impl<'de> Deserialize<'de> for RegexGINA {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value: &str = Deserialize::deserialize(deserializer)?;
    let value: anyhow::Result<RegexGINA> = RegexGINA::from_str(value);
    let value: RegexGINA = value.map_err(serde::de::Error::custom)?;
    Ok(value)
  }
}

impl Clone for RegexGINA {
  fn clone(&self) -> Self {
    RegexGINA::from_str(&self.raw).unwrap() // unwrap is safe since raw has been compiled before
  }
}

impl std::fmt::Debug for Conditions {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Conditions(len={})", self.0.len())?;
    Ok(())
  }
}

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
      .check(text, TEST_CHARACTER_NAME)
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
      assert!(rg.check(*text, TEST_CHARACTER_NAME).is_none());
    }
  }

  fn assert_pattern_matches(pattern: &str, text: &str, expectations: &[(&str, &str)]) {
    let rg = RegexGINA::from_str(pattern).expect("Invalid regex pattern");
    let result = rg
      .check(text, TEST_CHARACTER_NAME)
      .expect("RegexGINA did not match!");
    for (key, value) in expectations {
      assert_eq!(result[*key], *value);
    }
  }
}
