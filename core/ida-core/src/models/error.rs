use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeMap;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationError,
    ConfigInvalid,
    ConfigReadFailed,
    ConfigWriteFailed,
    PreferencesInvalid,
    PreferencesWriteFailed,
    SnapshotNotFound,
    SnapshotCorrupt,
    HistoryWriteFailed,
    AlertStateWriteFailed,
    FileIoError,
    ProviderNotFound,
    CodexNotFound,
    CodexUnauthenticated,
    WslNotFound,
    WslUnavailable,
    CaptureTimeout,
    CaptureFailed,
    ParserFailed,
    PartialSnapshot,
    DiscordNotConfigured,
    DiscordWebhookInvalid,
    DiscordDeliveryFailed,
    NotificationsUnavailable,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValidationError => "VALIDATION_ERROR",
            Self::ConfigInvalid => "CONFIG_INVALID",
            Self::ConfigReadFailed => "CONFIG_READ_FAILED",
            Self::ConfigWriteFailed => "CONFIG_WRITE_FAILED",
            Self::PreferencesInvalid => "PREFERENCES_INVALID",
            Self::PreferencesWriteFailed => "PREFERENCES_WRITE_FAILED",
            Self::SnapshotNotFound => "SNAPSHOT_NOT_FOUND",
            Self::SnapshotCorrupt => "SNAPSHOT_CORRUPT",
            Self::HistoryWriteFailed => "HISTORY_WRITE_FAILED",
            Self::AlertStateWriteFailed => "ALERT_STATE_WRITE_FAILED",
            Self::FileIoError => "FILE_IO_ERROR",
            Self::ProviderNotFound => "PROVIDER_NOT_FOUND",
            Self::CodexNotFound => "CODEX_NOT_FOUND",
            Self::CodexUnauthenticated => "CODEX_UNAUTHENTICATED",
            Self::WslNotFound => "WSL_NOT_FOUND",
            Self::WslUnavailable => "WSL_UNAVAILABLE",
            Self::CaptureTimeout => "CAPTURE_TIMEOUT",
            Self::CaptureFailed => "CAPTURE_FAILED",
            Self::ParserFailed => "PARSER_FAILED",
            Self::PartialSnapshot => "PARTIAL_SNAPSHOT",
            Self::DiscordNotConfigured => "DISCORD_NOT_CONFIGURED",
            Self::DiscordWebhookInvalid => "DISCORD_WEBHOOK_INVALID",
            Self::DiscordDeliveryFailed => "DISCORD_DELIVERY_FAILED",
            Self::NotificationsUnavailable => "NOTIFICATIONS_UNAVAILABLE",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub details: BTreeMap<String, String>,
    pub operation_id: String,
    pub occurred_at: DateTime<Utc>,
    pub retryable: bool,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: truncate(message.into(), 240),
            details: BTreeMap::new(),
            operation_id: Uuid::new_v4().to_string(),
            occurred_at: Utc::now(),
            retryable,
        }
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        if !crate::models::contains_sensitive_value(&value) {
            self.details.insert(key.into(), truncate(value, 240));
        }
        self
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationError, message, false)
    }

    pub fn file_io(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::FileIoError, message, true)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message, true)
    }

    pub fn parser_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ParserFailed, message, true)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for AppError {}

fn truncate(mut value: String, max_len: usize) -> String {
    if value.len() > max_len {
        value.truncate(max_len);
    }
    value
}
