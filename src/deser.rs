//! A handful of deserialisation helpers.
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

/// Deserialise a string or integer into a `u64`.
pub fn u64_from_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .ok_or_else(|| de::Error::custom("expected a u64 or a string"))
}

/// Deserialise a string or integer into a `u16`.
pub fn u16_from_str<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse::<u64>().ok()))
        .ok_or_else(|| de::Error::custom("expected a u64 or a string"))
        .and_then(|u| u16::try_from(u).map_err(|_| de::Error::custom("expected a u16")))
}

/// Deserialise a string or integer number of seconds since the Unix epoch into a datetime.
pub fn parse_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
        .and_then(|secs| NaiveDateTime::from_timestamp_opt(secs, 0))
        .map(|naive| DateTime::from_naive_utc_and_offset(naive, Utc))
        .ok_or_else(|| de::Error::custom("expected a timestamp in seconds"))
}

/// Deserialise a string as `Some(String)` and an empty string or null as `None`.
pub fn empty_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|s| s.filter(|s| !s.is_empty()))
}

/// Deserialise a single-element array into the element.
pub fn unit_array<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let Value::Array(arr) = serde_json::Value::deserialize(deserializer)? else {
        return Err(de::Error::custom("expected an array"));
    };
    if arr.len() != 1 {
        return Err(de::Error::custom("expected an array of length 1"));
    }
    T::deserialize(arr.into_iter().next().unwrap())
        .map_err(|e| de::Error::custom(format!("failed to deserialize: {}", e)))
}
