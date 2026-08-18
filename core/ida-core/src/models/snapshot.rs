use super::{
    validate_percentage, validate_provider_id, validate_safe_metadata, validate_schema_version,
    AppError, ErrorCode, SafeMetadata, SCHEMA_VERSION,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Ok,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMethod {
    Native,
    Wsl,
    Fixture,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SourcePlatform {
    Windows,
    Macos,
    Linux,
    Unknown,
}

impl SourcePlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum LimitStatus {
    Healthy,
    Watch,
    Low,
    Critical,
    Stale,
    Error,
}

impl LimitStatus {
    pub fn from_remaining_pct(remaining_pct: u8) -> Self {
        match remaining_pct {
            50..=100 => Self::Healthy,
            25..=49 => Self::Watch,
            10..=24 => Self::Low,
            _ => Self::Critical,
        }
    }

    pub fn severity_rank(self) -> u8 {
        match self {
            Self::Healthy => 0,
            Self::Watch => 1,
            Self::Low => 2,
            Self::Critical => 3,
            Self::Stale => 4,
            Self::Error => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessStatus {
    Fresh,
    Stale,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ProviderMetadata {
    pub account_label: Option<String>,
    pub raw_model_label: Option<String>,
    pub parser_version: String,
    pub raw_fields: SafeMetadata,
}

impl ProviderMetadata {
    pub fn new(parser_version: impl Into<String>) -> Self {
        Self {
            account_label: None,
            raw_model_label: None,
            parser_version: parser_version.into(),
            raw_fields: SafeMetadata::new(),
        }
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if let Some(account_label) = &self.account_label {
            validate_short_non_secret(account_label, 160, "account_label")?;
        }
        if let Some(raw_model_label) = &self.raw_model_label {
            validate_short_non_secret(raw_model_label, 80, "raw_model_label")?;
        }
        validate_short_non_secret(&self.parser_version, 80, "parser_version")?;
        validate_safe_metadata(&self.raw_fields, "provider")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct LimitWindow {
    pub id: String,
    pub label: String,
    pub window: String,
    pub remaining_pct: u8,
    pub used_pct: u8,
    pub resets_at: Option<DateTime<Utc>>,
    pub raw_reset_text: Option<String>,
    pub status: LimitStatus,
    pub status_reason: Option<String>,
    pub metadata: SafeMetadata,
}

impl LimitWindow {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        window: impl Into<String>,
        remaining_pct: u8,
        raw_reset_text: Option<String>,
    ) -> Result<Self, AppError> {
        validate_percentage(remaining_pct, "remaining_pct")?;
        let used_pct = 100 - remaining_pct;
        Ok(Self {
            id: id.into(),
            label: label.into(),
            window: window.into(),
            remaining_pct,
            used_pct,
            resets_at: None,
            raw_reset_text,
            status: LimitStatus::from_remaining_pct(remaining_pct),
            status_reason: None,
            metadata: SafeMetadata::new(),
        })
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.id.is_empty() || self.id.len() > 40 {
            return Err(AppError::validation(
                "Limit id is required and must be short.",
            ));
        }
        if self.label.is_empty() || self.label.len() > 40 {
            return Err(AppError::validation(
                "Limit label is required and must be short.",
            ));
        }
        validate_percentage(self.remaining_pct, "remaining_pct")?;
        validate_percentage(self.used_pct, "used_pct")?;
        if self.remaining_pct.saturating_add(self.used_pct) != 100 {
            return Err(AppError::validation(
                "remaining_pct and used_pct must add up to 100.",
            ));
        }
        if let Some(raw_reset_text) = &self.raw_reset_text {
            validate_short_non_secret(raw_reset_text, 120, "raw_reset_text")?;
        }
        if let Some(status_reason) = &self.status_reason {
            validate_short_non_secret(status_reason, 160, "status_reason")?;
        }
        validate_safe_metadata(&self.metadata, "limit")
    }

    pub fn as_stale(&self, reason: impl Into<String>) -> Self {
        let mut next = self.clone();
        next.status = LimitStatus::Stale;
        next.status_reason = Some(reason.into());
        next
    }

    pub fn as_error(&self, reason: impl Into<String>) -> Self {
        let mut next = self.clone();
        next.status = LimitStatus::Error;
        next.status_reason = Some(reason.into());
        next
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ProviderSnapshot {
    pub schema_version: u16,
    pub provider_id: String,
    pub provider_name: String,
    pub provider_status: ProviderStatus,
    pub scraped_at: DateTime<Utc>,
    pub capture_method: CaptureMethod,
    pub source_platform: SourcePlatform,
    pub limits: Vec<LimitWindow>,
    pub metadata: ProviderMetadata,
}

impl ProviderSnapshot {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_schema_version(self.schema_version, "ProviderSnapshot")?;
        validate_provider_id(&self.provider_id)?;
        if self.provider_name.is_empty() || self.provider_name.len() > 80 {
            return Err(AppError::validation("Provider name is required."));
        }
        if self.limits.is_empty() {
            return Err(AppError::validation(
                "Provider snapshot needs at least one limit.",
            ));
        }
        for limit in &self.limits {
            limit.validate()?;
        }
        self.metadata.validate()
    }

    pub fn new_codex(
        provider_status: ProviderStatus,
        scraped_at: DateTime<Utc>,
        capture_method: CaptureMethod,
        source_platform: SourcePlatform,
        limits: Vec<LimitWindow>,
        metadata: ProviderMetadata,
    ) -> Result<Self, AppError> {
        let snapshot = Self {
            schema_version: SCHEMA_VERSION,
            provider_id: "codex".to_string(),
            provider_name: "Codex".to_string(),
            provider_status,
            scraped_at,
            capture_method,
            source_platform,
            limits,
            metadata,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProviderReadResultType {
    Success,
    Partial,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ProviderReadResult {
    pub attempted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub provider_id: String,
    pub result_type: ProviderReadResultType,
    pub snapshot: Option<ProviderSnapshot>,
    pub error: Option<AppError>,
}

impl ProviderReadResult {
    pub fn success(
        attempted_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
        snapshot: ProviderSnapshot,
    ) -> Self {
        Self {
            attempted_at,
            completed_at: Some(completed_at),
            provider_id: snapshot.provider_id.clone(),
            result_type: ProviderReadResultType::Success,
            snapshot: Some(snapshot),
            error: None,
        }
    }

    pub fn partial(
        attempted_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
        snapshot: ProviderSnapshot,
        error: AppError,
    ) -> Self {
        Self {
            attempted_at,
            completed_at: Some(completed_at),
            provider_id: snapshot.provider_id.clone(),
            result_type: ProviderReadResultType::Partial,
            snapshot: Some(snapshot),
            error: Some(error),
        }
    }

    pub fn failure(
        attempted_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
        provider_id: impl Into<String>,
        error: AppError,
    ) -> Self {
        Self {
            attempted_at,
            completed_at: Some(completed_at),
            provider_id: provider_id.into(),
            result_type: ProviderReadResultType::Failure,
            snapshot: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AppState {
    pub schema_version: u16,
    pub provider_id: String,
    pub latest_snapshot: Option<ProviderSnapshot>,
    pub current_error: Option<AppError>,
    pub freshness_status: FreshnessStatus,
    pub last_attempted_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub next_poll_at: Option<DateTime<Utc>>,
    pub effective_limits: Vec<LimitWindow>,
}

impl AppState {
    pub fn unavailable(provider_id: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            provider_id: provider_id.into(),
            latest_snapshot: None,
            current_error: None,
            freshness_status: FreshnessStatus::Unavailable,
            last_attempted_at: None,
            last_success_at: None,
            next_poll_at: None,
            effective_limits: Vec::new(),
        }
    }

    pub fn lowest_status(&self) -> LimitStatus {
        self.effective_limits
            .iter()
            .map(|limit| limit.status)
            .max_by_key(|status| status.severity_rank())
            .unwrap_or(match self.freshness_status {
                FreshnessStatus::Fresh => LimitStatus::Healthy,
                FreshnessStatus::Stale => LimitStatus::Stale,
                FreshnessStatus::Unavailable | FreshnessStatus::Error => LimitStatus::Error,
            })
    }
}

fn validate_short_non_secret(value: &str, max_len: usize, field: &str) -> Result<(), AppError> {
    if value.len() > max_len {
        return Err(AppError::validation(format!("{field} is too long.")));
    }
    if super::contains_sensitive_value(value) {
        return Err(AppError::new(
            ErrorCode::ValidationError,
            format!("{field} contains sensitive-looking data."),
            false,
        ));
    }
    Ok(())
}
