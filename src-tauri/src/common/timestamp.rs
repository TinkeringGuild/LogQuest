use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::watch;

#[derive(Debug, Clone, PartialEq, Eq, ts_rs::TS)]
pub struct Timestamp(chrono::DateTime<chrono::Utc>);

impl Timestamp {
  pub fn now() -> Self {
    Self(chrono::Utc::now())
  }

  pub fn duration_until(&self, future_timestamp: &Timestamp) -> std::time::Duration {
    let delta = future_timestamp.0.signed_duration_since(self.0);
    delta.to_std().unwrap_or(std::time::Duration::ZERO)
  }
}

#[derive(Clone, ts_rs::TS)]
#[ts(type = "string")]
pub struct ObservableTimestamp(watch::Sender<Timestamp>, watch::Receiver<Timestamp>);

impl std::ops::Add<&super::duration::Duration> for &Timestamp {
  type Output = Timestamp;

  fn add(self, rhs: &super::duration::Duration) -> Self::Output {
    let rhs: std::time::Duration = rhs.clone().into();
    Timestamp(self.0.add(rhs))
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
    serializer.serialize_str(&self.to_string()) // uses the custom Display impl
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
    f.write_str(
      self
        .0
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        .as_str(),
    )?;
    Ok(())
  }
}

impl ObservableTimestamp {
  pub fn new(timestamp: Timestamp) -> Self {
    let (setter, getter) = watch::channel(timestamp);
    Self(setter, getter)
  }

  pub fn get(&self) -> watch::Ref<'_, Timestamp> {
    self.1.borrow()
  }

  pub fn set(&self, new_timestamp: Timestamp) {
    let _ = self.0.send(new_timestamp);
  }

  pub fn changed(
    &mut self,
  ) -> impl std::future::Future<Output = Result<(), watch::error::RecvError>> + '_ {
    self.1.changed()
  }
}

impl std::fmt::Debug for ObservableTimestamp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let timestamp = self.1.borrow();
    timestamp.fmt(f)
  }
}

impl Serialize for ObservableTimestamp {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let timestamp = self.1.borrow();
    timestamp.serialize(serializer)
  }
}
