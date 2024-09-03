use crate::matchers::MatchContext;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
  static ref TEMPLATE_VARS: Regex = Regex::new(r"\$\{\s*(\w+)\s*\}").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, ts_rs::TS)]
pub struct TemplateString(String);

impl Serialize for TemplateString {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.0)
  }
}

impl<'de> Deserialize<'de> for TemplateString {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let tmpl: String = Deserialize::deserialize(deserializer)?;
    Ok(Self(tmpl))
  }
}

impl TemplateString {
  pub fn tmpl(&self) -> &str {
    &self.0
  }

  pub fn render(&self, context: &MatchContext) -> String {
    TEMPLATE_VARS
      .replace_all(&self.0, |caps: &fancy_regex::Captures| {
        let var_name = caps
          .get(1)
          .expect("TEMPLATE_VARS should always capture a group 1 in replace_all")
          .as_str()
          .to_uppercase();
        if var_name == "C" {
          // This case not be necessary if it's ALWAYS added to named_values
          return context.character_name.clone();
        }
        if let Ok(group_number) = var_name.parse::<usize>() {
          if let Some(value) = context.group(group_number) {
            return value.to_owned();
          }
        } else {
          if let Some(value) = context.named_value(&var_name) {
            return value.to_owned();
          }
        }
        // Replace the var with an empty string if it's missing from the context
        String::new()
      })
      .into_owned()
  }
}

impl From<&str> for TemplateString {
  fn from(tmpl: &str) -> Self {
    TemplateString(tmpl.to_owned())
  }
}

impl From<String> for TemplateString {
  fn from(tmpl: String) -> Self {
    tmpl.as_str().into()
  }
}
