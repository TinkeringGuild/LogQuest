use regex::Regex;
use serde::{de::Visitor, Deserialize, Serialize, Serializer};
use std::str::FromStr as _;

#[derive(Debug, Clone)]
pub struct SerializableRegex(Regex);

impl Serialize for SerializableRegex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.0.as_str())
  }
}

// TODO: I think this can be massively simplified
impl<'de> Deserialize<'de> for SerializableRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct RegexVisitor;
    impl<'de> Visitor<'de> for RegexVisitor {
      type Value = SerializableRegex;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid regex pattern")
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Regex::from_str(value)
          .map(SerializableRegex)
          .map_err(serde::de::Error::custom)
      }
    }
    deserializer.deserialize_str(RegexVisitor)
  }
}
