use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time;

#[derive(Debug, Clone)]
pub struct Duration(u32);

impl Duration {
  pub fn from_millis(millis: u32) -> Self {
    Duration(millis)
  }
  pub fn from_secs(secs: u32) -> Self {
    Duration(secs * 1000)
  }
}

impl Into<time::Duration> for Duration {
  fn into(self) -> time::Duration {
    time::Duration::from_millis(self.0.into())
  }
}

impl Serialize for Duration {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_u32(self.0)
  }
}

impl<'de> Deserialize<'de> for Duration {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value: u32 = Deserialize::deserialize(deserializer)?;
    Ok(Duration(value))
  }
}
