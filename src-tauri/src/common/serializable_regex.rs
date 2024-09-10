use fancy_regex::Regex;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, ts_rs::TS)]
#[ts(type = "string")]
pub struct SerializableRegex {
  pub pattern: String,
  #[ts(skip)]
  pub compiled: Regex,
}

impl TryFrom<&str> for SerializableRegex {
  type Error = fancy_regex::Error;

  fn try_from(pattern: &str) -> Result<Self, Self::Error> {
    let compiled = Regex::new(pattern)?;
    let pattern = pattern.to_owned();
    Ok(Self { pattern, compiled })
  }
}

impl Eq for SerializableRegex {}
impl PartialEq for SerializableRegex {
  fn eq(&self, other: &Self) -> bool {
    self.pattern == other.pattern
  }
}

impl Serialize for SerializableRegex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.pattern)
  }
}

impl<'de> Deserialize<'de> for SerializableRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let pattern: String = Deserialize::deserialize(deserializer)?;
    let compiled = Regex::new(&pattern).map_err(serde::de::Error::custom)?;
    Ok(Self { pattern, compiled })
  }
}

#[cfg(test)]
mod test {
  use super::SerializableRegex;

  #[test]
  fn test_serde() {
    let before: SerializableRegex = "(?i)^This is only a test$".try_into().unwrap();
    let raw_json = serde_json::to_string(&before).expect("Could not serialize SerializableRegex!");
    let after: SerializableRegex =
      serde_json::from_str(&raw_json).expect("Could not deserialize SerializableRegex!");
    assert_eq!(before, after);
  }
}
