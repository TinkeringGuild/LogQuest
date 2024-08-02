use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct SerializableRegex(Regex);

impl Eq for SerializableRegex {}
impl PartialEq for SerializableRegex {
  fn eq(&self, other: &Self) -> bool {
    self.0.to_string() == other.0.to_string()
  }
}

impl From<Regex> for SerializableRegex {
  fn from(value: Regex) -> Self {
    Self(value)
  }
}

impl Serialize for SerializableRegex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.0.as_str())
  }
}

impl<'de> Deserialize<'de> for SerializableRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let regex_string: String = Deserialize::deserialize(deserializer)?;
    let regex = Regex::new(&regex_string).map_err(serde::de::Error::custom)?;
    Ok(Self(regex))
  }
}

#[cfg(test)]
mod test {
  use super::SerializableRegex;

  #[test]
  fn test_serde() {
    let before: SerializableRegex = regex::Regex::new("(?i)^This is only a test$")
      .unwrap()
      .into();
    let raw_json = serde_json::to_string(&before).expect("Could not serialize SerializableRegex!");
    let after: SerializableRegex =
      serde_json::from_str(&raw_json).expect("Could not deserialize SerializableRegex!");
    assert_eq!(before, after);
  }
}
