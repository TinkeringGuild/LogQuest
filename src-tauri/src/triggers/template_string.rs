use crate::matchers::MatchContext;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

lazy_static::lazy_static! {
  static ref TEMPLATE_VARS: Regex = Regex::new(r"\$\{\s*([\w_]+)\s*\}").unwrap();
}

#[derive(TS, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateString {
  tmpl: String,
  param_names: Vec<String>,
}

impl TemplateString {
  pub fn template(&self) -> &str {
    &self.tmpl
  }

  pub fn render(&self, context: &MatchContext) -> String {
    TEMPLATE_VARS
      .replace_all(&self.tmpl, |caps: &fancy_regex::Captures| {
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
    let param_names: Vec<String> = TEMPLATE_VARS
      .captures_iter(tmpl)
      // fancy_regex wraps Captures in a Result; TODO: how should this error case be handled?
      .filter_map(|c| c.ok())
      .filter_map(|captures| captures.get(1))
      .map(|mtch| mtch.as_str().to_owned())
      .collect();
    TemplateString {
      tmpl: tmpl.to_owned(),
      param_names,
    }
  }
}

impl From<String> for TemplateString {
  fn from(tmpl: String) -> Self {
    tmpl.as_str().into()
  }
}
