use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::database::DbError;

pub fn get_string_value(
    item: &HashMap<String, AttributeValue>,
    key: &str,
) -> Result<String, DbError> {
    item.get(key)
        .and_then(|av| av.as_s().ok().map(|s| s.to_owned()))
        .ok_or_else(|| DbError::Unknown(format!("{} not found in item", key)))
}

pub fn get_datetime_value(
    item: &HashMap<String, AttributeValue>,
    key: &str,
) -> Result<DateTime<Utc>, DbError> {
    let value = get_string_value(item, key)?;
    value
        .parse::<DateTime<Utc>>()
        .map_err(|e| DbError::Unknown(format!("Invalid {} format: {}", key, e)))
        .map(|dt| dt.with_timezone(&Utc))
}
