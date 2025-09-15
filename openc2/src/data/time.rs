use std::{
    fmt, ops,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

/// Epoch milliseconds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct DateTime(u64);

impl DateTime {
    /// Returns the current system time as a `DateTime`.
    ///
    /// # Panics
    /// This function panics if the system time is before the Unix epoch or does not fit in a `u64`.
    pub fn now() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_millis()
                .try_into()
                .expect("timestamp overflow"),
        )
    }

    pub fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

impl ops::Add<Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        DateTime(self.0 + rhs.0)
    }
}

impl ops::Sub for DateTime {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0.checked_sub(rhs.0).expect("rhs is earlier than lhs"))
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Duration(u64);

impl Duration {
    pub fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub fn from_secs(secs: u64) -> Self {
        Self(secs.checked_mul(1000).expect("duration overflow"))
    }

    pub fn from_hours(hours: u64) -> Self {
        Self(
            hours
                .checked_mul(60 * 60 * 1000)
                .expect("duration overflow"),
        )
    }

    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

impl ops::Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration(self.0.checked_add(rhs.0).expect("duration overflow"))
    }
}

impl ops::Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0.checked_sub(rhs.0).expect("rhs is longer than lhs"))
    }
}
