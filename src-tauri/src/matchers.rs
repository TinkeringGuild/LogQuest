use crate::common::serializable_regex::SerializableRegex;
use crate::gina::regex::RegexGINA;
use fancy_regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::error;
use ts_rs::TS;

#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Filter(Vec<Matcher>);

#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilterWithContext(Vec<MatcherWithContext>);

#[derive(Debug, Clone)]
pub struct MatchContext {
  pub group_values: Vec<Option<String>>,
  pub named_values: HashMap<String, String>,
  pub character_name: String,
}

#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Matcher {
  WholeLine(String),
  PartialLine(String),
  Pattern(SerializableRegex),
  GINA(RegexGINA),
}

/// The key difference between MatcherWithContext and Matcher is that some
/// MatcherWithContext variants store a String instead of a pre-compiled Regex.
/// This is because a WatchUntilFilterMatches effect might back-reference captures
/// from an earlier Regex, whose captured values must be interpolated escaped into
/// a JIT-compiled Regex that matches on those earlier values. One consequence of
/// this is that any parse error for a Regex String would only appear later when
/// creating a Timer, so these patterns should be validated at creation-time so
/// that the values stored in the Strings are guaranteed to be error-free.
#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatcherWithContext {
  WholeLine(String),
  PartialLine(String),
  Pattern(String),
  GINA(String),
}

impl From<Vec<Matcher>> for Filter {
  fn from(matchers: Vec<Matcher>) -> Self {
    Self(matchers)
  }
}

impl From<Vec<MatcherWithContext>> for FilterWithContext {
  fn from(matchers: Vec<MatcherWithContext>) -> Self {
    Self(matchers)
  }
}

impl Filter {
  pub fn check(&self, line: &str, character_name: &str) -> Option<MatchContext> {
    self
      .0
      .iter()
      .find_map(|matcher| matcher.check(line, character_name))
  }
}

impl FilterWithContext {
  pub fn compile_with_context(&self, context: &MatchContext) -> Filter {
    self
      .0
      .iter()
      .map(|matcher_with_context| {
        match matcher_with_context {
          MatcherWithContext::WholeLine(string) => Matcher::WholeLine(string.to_owned()),
          MatcherWithContext::PartialLine(string) => Matcher::PartialLine(string.to_owned()),
          MatcherWithContext::Pattern(pattern) => {
            // TODO: IMPLEMENT A CONTEXT-LOOKUP SYNTAX FOR LQ PATTERNS. THIS CODE IS JUST TEMPORARY
            let serializable_regex: SerializableRegex =
              pattern.as_str().try_into().unwrap_or_else(|_| {
                error!(r#"INVALID REGEX IN MatcherWithContext::Pattern("{pattern}")"#);
                "^(?!)$".try_into().unwrap() // unwrap is safe here
              });
            Matcher::Pattern(serializable_regex)
          }
          MatcherWithContext::GINA(pattern) => {
            // TODO: compile_with_context should probably have an error type that means
            // "partially successful" and still encapsulates a filter with the matchers
            // that didn't fail to convert. This is probably not very necessary considering
            // these patterns should be validated at compile time. The only way this conversion
            // might possibly happen would be importing an older version of a Triggers file
            // or something like that; since it's so unlikely this recovery logic isn't so bad.
            let regex_gina =
              RegexGINA::from_str_with_context(pattern, &context).unwrap_or_else(|_| {
                error!(r#"INVALID REGEX IN MatcherWithContext::GINA("{pattern}")"#);
                "^(?!)$".try_into().unwrap() // unwrap is safe here
              });
            Matcher::GINA(regex_gina)
          }
        }
      })
      .collect::<Vec<Matcher>>()
      .into()
  }
}

impl Matcher {
  pub fn gina(pattern: &str) -> Result<Self, fancy_regex::Error> {
    Ok(Self::GINA(pattern.try_into()?))
  }

  pub fn check(&self, line: &str, character_name: &str) -> Option<MatchContext> {
    match self {
      Self::WholeLine(exact_line) => {
        if line == exact_line {
          Some(MatchContext::empty(character_name))
        } else {
          None
        }
      }
      Self::PartialLine(substring) => {
        if line.contains(substring) {
          Some(MatchContext::empty(character_name))
        } else {
          None
        }
      }
      Self::Pattern(serializable_regex) => {
        let re: &Regex = &serializable_regex.compiled;
        if let Ok(Some(captures)) = re.captures(line) {
          Some(MatchContext::from_captures(&captures, re, character_name))
        } else {
          None
        }
      }
      Self::GINA(regex_gina) => regex_gina.check(line, character_name),
    }
  }
}

impl MatchContext {
  fn from_captures(captures: &Captures, re: &Regex, character_name: &str) -> Self {
    let group_values = captures
      .iter()
      .map(|option| option.map(|match_| match_.as_str().to_owned()))
      .collect();
    let named_values = re.capture_names().filter_map(|c| c).fold(
      HashMap::<String, String>::new(),
      |mut memo, name| {
        if let Some(match_) = captures.name(name) {
          memo.insert(name.to_uppercase(), match_.as_str().to_owned());
        }
        memo
      },
    );
    Self {
      group_values,
      named_values,
      character_name: character_name.to_owned(),
    }
  }

  pub fn group(&self, index: usize) -> Option<&str> {
    if let Some(value) = self.group_values.get(index) {
      value.as_deref()
    } else {
      None
    }
  }

  pub fn named_value(&self, key: &str) -> Option<&str> {
    self
      .named_values
      .get(&key.to_uppercase())
      .map(|s| s.as_str())
  }

  pub fn empty(character_name: &str) -> Self {
    Self {
      group_values: Vec::with_capacity(0),
      named_values: HashMap::with_capacity(0),
      character_name: character_name.to_owned(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{Filter, Matcher, MatcherWithContext};
  use crate::matchers::FilterWithContext;

  #[test]
  fn test_gina_matchers_with_context() {
    let toon = "Xenk";
    let first_matcher: Filter = vec![Matcher::GINA(
      r"^(\w+) (hits YOU for (\d+) points? of damage|tries to hit YOU)"
        .try_into()
        .unwrap(),
    )]
    .into();

    let context = first_matcher
      .check("Bristlebane hits YOU for 1000 points of damage", toon)
      .expect("Regex did not match!");

    let filter_with_context: FilterWithContext = vec![MatcherWithContext::GINA(
      r"^${1} has been slain by (?<whom>{C})".try_into().unwrap(),
    )]
    .into();

    let compiled_filter_with_context = filter_with_context.compile_with_context(&context);

    // This API is a little weird because the character name is provided in the context and as a param to check
    let next_context = compiled_filter_with_context
      .check(&format!("Bristlebane has been slain by {toon}"), toon)
      .unwrap();

    let by_group_number = next_context
      .group(1)
      .expect("Failed to access group by number");
    let by_group_name = next_context
      .named_value("wHoM")
      .expect("Failed to access group by name");
    let by_character_name = next_context
      .named_value("C")
      .expect("Failed to access character name");

    assert_eq!(by_group_number, toon);
    assert_eq!(by_group_name, toon);
    assert_eq!(by_character_name, toon);
  }
}
