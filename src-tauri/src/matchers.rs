use crate::common::serializable_regex::SerializableRegex;
use crate::gina::regex::RegexGINA;
use serde::{Deserialize, Serialize};

pub type Filter = Vec<Matcher>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Matcher {
  WholeLine(String),
  PartialLine(String),
  Pattern(SerializableRegex),
  GINA(RegexGINA),
}

impl Matcher {
  pub fn gina(pattern: &str) -> anyhow::Result<Self> {
    Ok(Self::GINA(RegexGINA::from_str(pattern)?))
  }
}
