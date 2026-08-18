mod config;
mod error;
mod snapshot;

pub use config::*;
pub use error::*;
pub use snapshot::*;

use std::collections::BTreeMap;

pub type SafeMetadata = BTreeMap<String, String>;

pub const SCHEMA_VERSION: u16 = 1;

pub fn validate_schema_version(value: u16, model: &str) -> Result<(), AppError> {
    if value == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::ValidationError,
            format!("{model} schema version is unsupported."),
            false,
        )
        .with_detail("expected", SCHEMA_VERSION.to_string())
        .with_detail("actual", value.to_string()))
    }
}

pub fn validate_provider_id(provider_id: &str) -> Result<(), AppError> {
    let valid = !provider_id.is_empty()
        && provider_id.len() <= 40
        && provider_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_');

    if valid {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::ValidationError,
            "Provider id must be a lowercase slug.",
            false,
        ))
    }
}

pub fn validate_percentage(value: u8, field: &str) -> Result<(), AppError> {
    if value <= 100 {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::ValidationError,
            format!("{field} must be between 0 and 100."),
            false,
        ))
    }
}

pub fn validate_safe_metadata(metadata: &SafeMetadata, context: &str) -> Result<(), AppError> {
    for (key, value) in metadata {
        let combined = format!("{key} {value}");
        if contains_sensitive_value(&combined) {
            return Err(AppError::new(
                ErrorCode::ValidationError,
                format!("{context} metadata contains sensitive-looking data."),
                false,
            )
            .with_detail("field", key.clone()));
        }
        if value.len() > 500 {
            return Err(AppError::new(
                ErrorCode::ValidationError,
                format!("{context} metadata value is too long."),
                false,
            )
            .with_detail("field", key.clone()));
        }
    }

    Ok(())
}

pub fn contains_sensitive_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let sensitive_markers = [
        "discord.com/api/webhooks",
        "discordapp.com/api/webhooks",
        "webhook_url",
        "authorization:",
        "bearer ",
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "openai_api_key",
        "anthropic_api_key",
        "sk-",
        "xoxb-",
        "ghp_",
        "gho_",
    ];

    sensitive_markers
        .iter()
        .any(|marker| lower.contains(marker))
}
