use crate::common::random_id;
use crate::matchers::MatchContext;
use fancy_regex::{Captures, Regex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, LinkedList};
use tracing::warn;

lazy_static::lazy_static! {
  /// For extracting out GINA variable placeholders from a GINA regex (e.g. {S}, {N>100}, {C}, {S2})
  static ref REGEX_VARS: Regex =
    Regex::new(r"\{\s*(?:([Cc]|[Ss]\d*)|(?:([Nn]\d*)\s*(?:(>=|<=|=|>|<)\s*(-?\d+))?))\s*\}").unwrap();

  /// A MatcherWithContext::GINA can use patterns like ${1} to back-reference a capture in the Trigger's initial regex
  static ref REGEX_REFERENCES: Regex =
      Regex::new(r"\$\{\s*(\d+|[A-Za-z_]\w*)\s*\}").unwrap();

  /// Named capture groups are injected into the converted Regex; this matches the generated names
  static ref GENERATED_NAMED_CAPTURE_NAME: Regex = Regex::new(r"^LQ[A-Z0-9]{8}$").unwrap();

  /// Used by fix_possibly_invalid_character_classes to match text inside character classes
  static ref CHARACTER_CLASS_CONTENTS: Regex = Regex::new(r"\[([^\]]+?(?<!\\))\]").unwrap();

  /// Used by fix_possibly_invalid_character_classes to match '-' characters inside character classes
  /// A "-" at the beginning or end of the character class is always a valid literal "-" so it's useful
  /// that this regex matches the characters surrounding the dash. With fancy_regex, if there is a space
  /// AFTER a dash and a letter character before the dash, it is considered an invalid range (however a
  /// space before a dash always means a literal dash).
  static ref CHARACTER_RANGE_DASH: Regex = Regex::new(r"([^\\])-(.)(?<! -[A-Za-z0-9])").unwrap();

  /// Used by fix_possibly_invalid_character_classes to detect valid character classes
  static ref VALID_CHARACTER_RANGE: Regex = Regex::new(r"(?<!\\)(?:[A-Z]-[A-Z]|[a-z]-[a-z]|[0-9]-[0-9])").unwrap();

}

type ConditionsList = LinkedList<Box<dyn Fn(&Captures) -> bool + Send + Sync + 'static>>;
struct Conditions(ConditionsList);

#[derive(Debug, ts_rs::TS)]
#[ts(type = "string")]
pub struct RegexGINA {
  pub raw: String,
  #[ts(skip)]
  compiled: Regex,
  #[ts(skip)]
  named_projections: HashMap<String, String>,
  #[ts(skip)]
  positional_projections: Vec<usize>,
  #[ts(skip)]
  conditions: Conditions,
}

impl TryFrom<&str> for RegexGINA {
  type Error = fancy_regex::Error;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    Self::from_str(value)
  }
}

impl RegexGINA {
  pub fn from_str(pattern: &str) -> Result<Self, fancy_regex::Error> {
    let fixed = Self::fix_possibly_invalid_character_classes(pattern);
    Self::from_str_without_fixing_character_classes(&fixed)
  }

  // A lot of the complexity here comes from how special GINA tokens
  // like {S} or {N>=10} are extracted using named capture groups, but
  // the inclusion of these capture groups must be invisible to the
  // end-user writing a Filter; notably, they should be able to address
  // their regex's capture groups by index without these dynamically
  // interpolated capture groups affecting the indices they'd expect.
  pub fn from_str_without_fixing_character_classes(
    pattern: &str,
  ) -> Result<Self, fancy_regex::Error> {
    let mut named_projections: HashMap<String, String> = HashMap::new();

    let mut conditions: ConditionsList = LinkedList::new();
    let with_replacements = REGEX_VARS.replace_all(pattern, |captures: &Captures| {
      let projected_from = generate_named_capture_name();

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
        if is_generated_capture_name(capture_name) {
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

  pub fn from_str_with_context(
    pattern: &str,
    context: &MatchContext,
  ) -> Result<Self, fancy_regex::Error> {
    let processed_pattern = Self::interpolate_escaped_context_variables(pattern, context);
    Self::from_str(&processed_pattern)
  }

  pub fn from_str_with_context_without_fixing_character_classes(
    pattern: &str,
    context: &MatchContext,
  ) -> Result<Self, fancy_regex::Error> {
    let processed_pattern = Self::interpolate_escaped_context_variables(pattern, context);
    Self::from_str_without_fixing_character_classes(&processed_pattern)
  }

  fn interpolate_escaped_context_variables(pattern: &str, context: &MatchContext) -> String {
    REGEX_REFERENCES
      .replace_all(pattern, |captures: &Captures| {
        if let Some(group_reference_match) = captures.get(1) {
          let group_reference = group_reference_match.as_str();
          if let Ok(group_number) = group_reference.parse::<usize>() {
            if let Some(group_value) = context.group(group_number) {
              return fancy_regex::escape(group_value).into_owned();
            }
          } else {
            if let Some(group_value) = context.named_value(group_reference) {
              return fancy_regex::escape(group_value).into_owned();
            }
          }
        }
        String::new()
      })
      .into_owned()
  }

  /// Returns a MatchContext if the RegexGINA matches. A character name must be passed in
  /// because the regex could have a {C} token.
  pub fn check(&self, line: &str, character_name: &str) -> Option<MatchContext> {
    let direct_captures: Captures = match self.compiled.captures(line) {
      Ok(Some(captures)) => captures,
      Ok(None) => return None,
      Err(_) => return None,
    };

    for condition in self.conditions.0.iter() {
      if !condition(&direct_captures) {
        return None;
      }
    }

    let mut named_values = HashMap::<String, String>::new();
    for (capture_name, output_name) in self.named_projections.iter() {
      let Some(captured_value) = direct_captures
        .name(&capture_name)
        .and_then(|m| Some(m.as_str().to_owned()))
      else {
        continue;
      };
      if let Some(replaced_value) =
        named_values.insert(output_name.to_uppercase(), captured_value.clone())
      {
        // Make sure all values for a given named capture are equal
        if replaced_value != captured_value {
          return None;
        }
      }
    }

    if let Some(captured_character_name) = named_values.get("C") {
      if captured_character_name != character_name {
        return None;
      }
    }

    let group_values: Vec<Option<String>> = self
      .positional_projections
      .iter()
      .map(|i| direct_captures.get(*i).map(|m| m.as_str().to_owned()))
      .collect();

    Some(MatchContext {
      group_values,
      named_values,
      character_name: character_name.to_owned(),
    })
  }

  fn pattern_for_number_capture(capture_name: &str) -> String {
    format!(r"(?<{capture_name}>-?\d+)")
  }

  fn pattern_for_string_capture(capture_name: &str) -> String {
    format!(r"(?<{capture_name}>.+)") // TODO: Should this be lazy? (i.e. /.+?/)
  }

  fn pattern_for_character_name_capture(capture_name: &str) -> String {
    format!(r"(?<{}>{})", capture_name, r"[A-Za-z]{3,15}") // On P99, 3-letter toon names do exist
  }

  fn create_condition_for_numeric_constraints(
    operator: String,
    operand: i64,
    projected_from: String,
  ) -> impl Fn(&Captures) -> bool + Send + 'static {
    move |caps: &Captures| {
      Self::check_numeric_constraints(operator.clone(), operand, projected_from.clone(), caps)
    }
  }

  fn check_numeric_constraints(
    operator: String,
    operand: i64,
    projected_from: String,
    caps: &Captures,
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

  /// GINA seems to allow a "-" character in character classes that are invalid in other Regex engines,
  /// for example "[\w`-']". The "-" is used for expressing ranges of characters (A-Z, a-z, or 0-9), but
  /// to match a literal "-" you are supposed to escape it, put the dash at the beginning/end of the
  /// character class, or put a space before/after the "-". Apparently fancy_regex does NOT treat a dash
  /// with a space AFTER it as a valid literal "-", so these cases must be escaped too.
  ///
  /// This problem with character classes seems to be a common enough problem in GINA triggers that
  /// having an auto-fixer for this is crucial for a proper GINA import. This function takes in a regex
  /// pattern and escapes any of these invalid "-" characters in character classes.
  fn fix_possibly_invalid_character_classes(pattern: &str) -> String {
    let all_fixed = CHARACTER_CLASS_CONTENTS.replace_all(&pattern, |cc_captures: &Captures| {
      let whole_match = cc_captures.get(0).unwrap().as_str().to_owned(); // unwrap should be safe here

      let Some(inner_match) = cc_captures.get(1) else {
        return whole_match;
      };

      let character_class_contents = inner_match.as_str();
      // Assume there is only 1 invalid dash inside the character class
      if character_class_contents.len() <= 2 || character_class_contents.starts_with("-") || character_class_contents.ends_with("-") {
        return whole_match;
      }

      let fixed_contents = CHARACTER_RANGE_DASH.replace_all(character_class_contents, |surrounded_dash_caps: &Captures| {
        let surrounded_dash = surrounded_dash_caps.get(0).unwrap().as_str();
        if let Ok(true) = VALID_CHARACTER_RANGE.is_match(surrounded_dash) {
          return surrounded_dash.to_owned();
        }
        let (Some(match_before), Some(match_after)) = (surrounded_dash_caps.get(1), surrounded_dash_caps.get(2)) else {
          return surrounded_dash.to_owned();
        };
        format!("{}\\-{}", match_before.as_str(), match_after.as_str())
      });
      if fixed_contents != character_class_contents {
        warn!("Detected a GINA regex with an invalid character class. Changing /{whole_match}/ to /[{fixed_contents}]/");
      }
      format!("[{fixed_contents}]")
    });
    all_fixed.into_owned()
  }
}

// This must be consistent with GENERATED_NAMED_CAPTURE_NAME
fn generate_named_capture_name() -> String {
  format!("LQ{}", random_id(8)) // 8 makes chance of two collisions approx 1/2.8e12
}

fn is_generated_capture_name(capture_name: &str) -> bool {
  GENERATED_NAMED_CAPTURE_NAME
    .is_match(capture_name)
    .is_ok_and(|boolean| boolean)
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
    let value: String = Deserialize::deserialize(deserializer)?;
    let value = value.as_str().try_into();
    let value: RegexGINA = value.map_err(serde::de::Error::custom)?;
    Ok(value)
  }
}

impl Clone for RegexGINA {
  fn clone(&self) -> Self {
    self.raw.as_str().try_into().unwrap() // unwrap is safe since raw has been compiled before
  }
}

impl Eq for RegexGINA {}
impl PartialEq for RegexGINA {
  fn eq(&self, other: &Self) -> bool {
    self.raw == other.raw
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

  const TOON: &str = "Xenk";

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
    {
      let context = create_context(
        r"^You (must target an NPC to taunt first\.|taunt {S} to ignore others and attack you\!)$",
        "You must target an NPC to taunt first.",
      );
      assert_eq!(context.named_value("S"), None);
    }
    {
      let context = create_context(
        r"^You (must target an NPC to taunt first\.|taunt {S} to ignore others and attack you\!)$",
        "You taunt Bristlebane to ignore others and attack you!",
      );
      assert_eq!(context.named_value("S").unwrap(), "Bristlebane");
    }
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
  fn test_accessing_captures_by_index_and_name() {
    let context = create_context(
      r"^Here're some words: (\w+), (\w+), (?<conjunction>\w+) (\w+)$",
      "Here're some words: one, two, and three",
    );
    assert_eq!(context.group(1).unwrap(), "one");
    assert_eq!(context.group(2).unwrap(), "two");
    assert_eq!(context.group(4).unwrap(), "three");
    assert_eq!(context.named_value("conjunction").unwrap(), "and");
    assert_eq!(context.named_value("CONJUNCTION").unwrap(), "and");
  }

  #[test]
  fn test_optional_positional_captures() {
    {
      let context = create_context(
        r"^There are (?:(\w+) parameters (\w+)|none) here$",
        "There are none here",
      );
      assert_eq!(context.group(1), None);
    }
    {
      let context = create_context(
        r"^There are (?:(\w+) parameters (\w+)|none) here$",
        "There are two parameters right here",
      );
      assert_eq!(context.group(1).unwrap(), "two");
      assert_eq!(context.group(2).unwrap(), "right");
    }
  }

  #[test]
  fn test_compile_with_context_using_context_references() {
    let context = create_context("^This is a (?<named>capture)", "This is a capture");
    let re = RegexGINA::from_str_with_context(r"^Get ${named}", &context).unwrap();
    assert!(re.check("Get capture", TOON).is_some());
    assert!(re.check("Get something else", TOON).is_none());

    let context = create_context(
      r"^([\w -'`]+)\'s body pulses with mystic fortitude\.$",
      "Xenk's body pulses with mystic fortitude.",
    );
    let re =
      RegexGINA::from_str_with_context(r"^${1} has been slain by (?>[^!]+)\!$", &context).unwrap();
    assert!(re
      .check("Xenk has been slain by Vulak`Aerr!", TOON)
      .is_some());
    assert!(re
      .check("Goner has been slain by Vulak`Aerr!", TOON)
      .is_none());
  }

  #[test]
  fn test_fixing_gina_regex_with_invalid_character_class() {
    fn assert_unmodified(input: &str) {
      let re = RegexGINA::from_str(input).expect("assert_unmodified compile error");
      assert_eq!(re.raw, input);
    }
    fn assert_fix(input: &str, output: &str) {
      let re = RegexGINA::from_str(input).expect("assert_fix compile error");
      assert_eq!(re.raw, output);
    }

    assert_fix(
      r"^You have stolen ([\w\s-`']+)[.]?$",
      r"^You have stolen ([\w\s\-`']+)[.]?$",
    );
    assert_fix(
      r"^([`'\w- ]+) has been slain",
      r"^([`'\w\- ]+) has been slain",
    );
    assert_fix(
      r"^([`' -\w]+) has been slain",
      r"^([`' \-\w]+) has been slain",
    );
    assert_fix(
      r"^Your target resisted the ([\w -'`]+)(?<!LowerElement) spell\.$",
      r"^Your target resisted the ([\w \-'`]+)(?<!LowerElement) spell\.$",
    );
    assert_fix(r"[a- ][a- ]", r"[a\- ][a\- ]");

    assert_unmodified(r"([-]+)");
    assert_unmodified(r"^([A-Za-z]+) has been slain");
    assert_unmodified(r"^([-`'\w ]+) has been slain");
    assert_unmodified(r"^([ `'\w-]+) has been slain");
    assert_unmodified(r"^([-A-Za-z0-9`']+) has been slain");
    assert_unmodified(r"^([`'A-Za-z0-9-]+) has been slain");
    assert_unmodified(r"^([`\-'A-Za-z0-9]+) has been slain");
    assert_unmodified(r"^([\-`'A-Za-z0-9]+) has been slain");
    assert_unmodified(r"^([`'A-Za-z0-9\-]+) has been slain");
  }

  fn create_context(pattern: &str, text: &str) -> MatchContext {
    let regex_gina: RegexGINA = pattern
      .try_into()
      .expect("create_context could not compile RegexGINA pattern!");
    regex_gina
      .check(text, TOON)
      .expect("create_context pattern and text do not match!")
  }

  fn assert_pattern_does_not_match(pattern: &str, texts: &[&str]) {
    let regex_gina: RegexGINA = pattern.try_into().expect("Invalid regex pattern");
    for text in texts {
      assert!(regex_gina.check(*text, TOON).is_none());
    }
  }

  fn assert_pattern_matches(pattern: &str, text: &str, expectations: &[(&str, &str)]) {
    let regex_gina: RegexGINA = pattern.try_into().expect("Invalid regex pattern");
    let result = regex_gina
      .check(text, TOON)
      .expect("RegexGINA did not match!");
    for (key, value) in expectations {
      assert_eq!(result.named_value(key).unwrap(), *value);
    }
  }
}
