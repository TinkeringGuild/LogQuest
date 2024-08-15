//! This Duration type is needed for custom serialization of time-related integers
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

/// Wrapper around an integer representing milliseconds.
/// This is mainly useful for serialization of Durations.
/// Since this is a backed by a u32, the longest possible
/// duration would be 49.71 days, which is fine for LogQuest.
#[derive(TS, Debug, Clone, PartialEq, Eq)]
pub struct Duration(pub u32);

impl Duration {
  pub fn from_millis(millis: u32) -> Self {
    Duration(millis)
  }
  pub fn from_secs(secs: u32) -> Self {
    Duration(secs * 1000)
  }
}

impl From<Duration> for std::time::Duration {
  fn from(value: Duration) -> Self {
    std::time::Duration::from_millis(value.0.into())
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
