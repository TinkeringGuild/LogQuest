use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

#[derive(TS, Debug, Clone, PartialEq, Eq)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);

impl Timestamp {
  pub fn now() -> Self {
    Self(chrono::Utc::now())
  }
}

impl From<chrono::NaiveDateTime> for Timestamp {
  fn from(naive: chrono::NaiveDateTime) -> Self {
    Self(naive.and_utc())
  }
}

impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
  fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
    Self(value)
  }
}

impl From<Timestamp> for chrono::DateTime<chrono::Utc> {
  fn from(value: Timestamp) -> Self {
    value.0
  }
}

impl Serialize for Timestamp {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_str())
  }
}

impl<'de> Deserialize<'de> for Timestamp {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let dt = chrono::DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
    Ok(Self(dt.with_timezone(&chrono::Utc)))
  }
}

impl std::fmt::Display for Timestamp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.0.to_rfc3339().as_str())?;
    Ok(())
  }
}
