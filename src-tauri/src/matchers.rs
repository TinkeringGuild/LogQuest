use crate::common::serializable_regex::SerializableRegex;
use serde::{Deserialize, Serialize};

pub type Filter = Vec<Matcher>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Matcher {
  WholeLine(String),
  PartialLine(String),
  Pattern(SerializableRegex),
  GINAPattern(String),
}
