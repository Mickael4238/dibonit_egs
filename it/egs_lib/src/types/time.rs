//! Time utilities for EGS
//!
//! Provides time-related utilities and types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// A timestamp wrapper for EGS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgsTimestamp(DateTime<Utc>);

impl EgsTimestamp {
    /// Creates a new timestamp with the current time
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Creates a timestamp from a DateTime
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Returns the underlying DateTime
    pub fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }

    /// Returns the timestamp as RFC 3339 string
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }

    /// Checks if the timestamp is within the given duration from now
    pub fn is_recent(&self, duration: Duration) -> bool {
        let now = Utc::now();
        now - self.0 <= duration
    }

    /// Returns the age of the timestamp
    pub fn age(&self) -> Duration {
        Utc::now() - self.0
    }
}

impl From<DateTime<Utc>> for EgsTimestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<EgsTimestamp> for DateTime<Utc> {
    fn from(ts: EgsTimestamp) -> Self {
        ts.0
    }
}

/// A duration wrapper for EGS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgsDuration(Duration);

impl EgsDuration {
    /// Creates a new duration from seconds
    pub fn from_secs(secs: i64) -> Self {
        Self(Duration::seconds(secs))
    }

    /// Creates a new duration from milliseconds
    pub fn from_millis(millis: i64) -> Self {
        Self(Duration::milliseconds(millis))
    }

    /// Returns the underlying Duration
    pub fn as_duration(&self) -> &Duration {
        &self.0
    }

    /// Returns the duration in seconds
    pub fn as_secs(&self) -> i64 {
        self.0.num_seconds()
    }

    /// Returns the duration in milliseconds
    pub fn as_millis(&self) -> i64 {
        self.0.num_milliseconds()
    }
}

impl From<Duration> for EgsDuration {
    fn from(d: Duration) -> Self {
        Self(d)
    }
}

impl From<EgsDuration> for Duration {
    fn from(d: EgsDuration) -> Self {
        d.0
    }
}
