use super::{validate_provider_id, validate_schema_version, AppError, ErrorCode, SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use specta::Type;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    NativeThenWsl,
    NativeOnly,
    WslOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AppConfig {
    pub schema_version: u16,
    pub active_provider_id: String,
    pub polling_interval_seconds: u64,
    pub stale_after_seconds: u64,
    pub alert_thresholds: Vec<u8>,
    pub native_notifications_enabled: bool,
    pub discord_alerts_enabled: bool,
    pub discord_webhook_url: Option<String>,
    pub capture_mode: CaptureMode,
    pub history_retention_hours: u64,
    pub log_level: LogLevel,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            active_provider_id: "codex".to_string(),
            polling_interval_seconds: 900,
            stale_after_seconds: 1800,
            alert_thresholds: vec![75, 50, 25, 10, 5],
            native_notifications_enabled: true,
            discord_alerts_enabled: false,
            discord_webhook_url: None,
            capture_mode: CaptureMode::NativeThenWsl,
            history_retention_hours: 24,
            log_level: LogLevel::Info,
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_schema_version(self.schema_version, "AppConfig")?;
        validate_provider_id(&self.active_provider_id)?;
        validate_polling_interval(self.polling_interval_seconds)?;
        validate_stale_after(self.stale_after_seconds)?;
        validate_alert_thresholds(&self.alert_thresholds)?;
        if let Some(webhook_url) = &self.discord_webhook_url {
            validate_discord_webhook_url(webhook_url)?;
        }
        if !(1..=168).contains(&self.history_retention_hours) {
            return Err(AppError::validation(
                "History retention must be between 1 and 168 hours.",
            ));
        }
        Ok(())
    }

    pub fn redacted(&self, config_path: Option<String>) -> AppConfigRedacted {
        AppConfigRedacted {
            schema_version: self.schema_version,
            active_provider_id: self.active_provider_id.clone(),
            polling_interval_seconds: self.polling_interval_seconds,
            stale_after_seconds: self.stale_after_seconds,
            alert_thresholds: self.alert_thresholds.clone(),
            native_notifications_enabled: self.native_notifications_enabled,
            discord_alerts_enabled: self.discord_alerts_enabled,
            discord_webhook_configured: self.discord_webhook_url.is_some(),
            discord_webhook_masked: self
                .discord_webhook_url
                .as_ref()
                .map(|_| "https://discord.com/api/webhooks/...".to_string()),
            capture_mode: self.capture_mode,
            history_retention_hours: self.history_retention_hours,
            log_level: self.log_level,
            config_path,
        }
    }

    pub fn apply_patch(
        &self,
        patch: AppConfigPatch,
        secret_updates: Option<SecretUpdates>,
    ) -> Result<Self, AppError> {
        let mut next = self.clone();

        if let Some(active_provider_id) = patch.active_provider_id {
            next.active_provider_id = active_provider_id;
        }
        if let Some(polling_interval_seconds) = patch.polling_interval_seconds {
            next.polling_interval_seconds = polling_interval_seconds;
        }
        if let Some(stale_after_seconds) = patch.stale_after_seconds {
            next.stale_after_seconds = stale_after_seconds;
        }
        if let Some(alert_thresholds) = patch.alert_thresholds {
            next.alert_thresholds = alert_thresholds;
        }
        if let Some(native_notifications_enabled) = patch.native_notifications_enabled {
            next.native_notifications_enabled = native_notifications_enabled;
        }
        if let Some(discord_alerts_enabled) = patch.discord_alerts_enabled {
            next.discord_alerts_enabled = discord_alerts_enabled;
        }
        if let Some(capture_mode) = patch.capture_mode {
            next.capture_mode = capture_mode;
        }
        if let Some(history_retention_hours) = patch.history_retention_hours {
            next.history_retention_hours = history_retention_hours;
        }
        if let Some(log_level) = patch.log_level {
            next.log_level = log_level;
        }
        if let Some(secret_updates) = secret_updates {
            if let Some(discord_webhook_url) = secret_updates.discord_webhook_url {
                let trimmed = discord_webhook_url.trim();
                next.discord_webhook_url = if trimmed.is_empty() {
                    None
                } else {
                    validate_discord_webhook_url(trimmed)?;
                    Some(trimmed.to_string())
                };
            }
        }

        next.validate()?;
        Ok(next)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AppConfigRedacted {
    pub schema_version: u16,
    pub active_provider_id: String,
    pub polling_interval_seconds: u64,
    pub stale_after_seconds: u64,
    pub alert_thresholds: Vec<u8>,
    pub native_notifications_enabled: bool,
    pub discord_alerts_enabled: bool,
    pub discord_webhook_configured: bool,
    pub discord_webhook_masked: Option<String>,
    pub capture_mode: CaptureMode,
    pub history_retention_hours: u64,
    pub log_level: LogLevel,
    pub config_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AppConfigPatch {
    pub active_provider_id: Option<String>,
    pub polling_interval_seconds: Option<u64>,
    pub stale_after_seconds: Option<u64>,
    pub alert_thresholds: Option<Vec<u8>>,
    pub native_notifications_enabled: Option<bool>,
    pub discord_alerts_enabled: Option<bool>,
    pub capture_mode: Option<CaptureMode>,
    pub history_retention_hours: Option<u64>,
    pub log_level: Option<LogLevel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SecretUpdates {
    pub discord_webhook_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SecretPresence {
    pub discord_webhook_url: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WidgetPreferences {
    pub schema_version: u16,
    pub visible_on_launch: bool,
    pub always_on_top: bool,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub width: u32,
    pub height: u32,
    pub display_id: Option<String>,
}

impl Default for WidgetPreferences {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            visible_on_launch: true,
            always_on_top: true,
            position_x: None,
            position_y: None,
            width: 280,
            height: 160,
            display_id: None,
        }
    }
}

impl WidgetPreferences {
    pub fn validate(&self) -> Result<(), AppError> {
        validate_schema_version(self.schema_version, "WidgetPreferences")?;
        if !(280..=800).contains(&self.width) {
            return Err(AppError::validation(
                "Widget width must be between 280 and 800.",
            ));
        }
        if !(160..=600).contains(&self.height) {
            return Err(AppError::validation(
                "Widget height must be between 160 and 600.",
            ));
        }
        if let Some(display_id) = &self.display_id {
            if display_id.len() > 160 || super::contains_sensitive_value(display_id) {
                return Err(AppError::validation("Display id is invalid."));
            }
        }
        Ok(())
    }

    pub fn apply_patch(&self, patch: WidgetPreferencesPatch) -> Result<Self, AppError> {
        let mut next = self.clone();
        if let Some(visible_on_launch) = patch.visible_on_launch {
            next.visible_on_launch = visible_on_launch;
        }
        if let Some(always_on_top) = patch.always_on_top {
            next.always_on_top = always_on_top;
        }
        if let Some(position_x) = patch.position_x {
            next.position_x = position_x;
        }
        if let Some(position_y) = patch.position_y {
            next.position_y = position_y;
        }
        if let Some(width) = patch.width {
            next.width = width;
        }
        if let Some(height) = patch.height {
            next.height = height;
        }
        if let Some(display_id) = patch.display_id {
            next.display_id = display_id;
        }
        next.validate()?;
        Ok(next)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WidgetPreferencesPatch {
    pub visible_on_launch: Option<bool>,
    pub always_on_top: Option<bool>,
    pub position_x: Option<Option<i32>>,
    pub position_y: Option<Option<i32>>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub display_id: Option<Option<String>>,
}

pub fn validate_polling_interval(value: u64) -> Result<(), AppError> {
    if (60..=86_400).contains(&value) {
        Ok(())
    } else {
        Err(AppError::validation(
            "Polling interval must be between 60 and 86400 seconds.",
        ))
    }
}

pub fn validate_stale_after(value: u64) -> Result<(), AppError> {
    if (120..=172_800).contains(&value) {
        Ok(())
    } else {
        Err(AppError::validation(
            "Stale threshold must be between 120 and 172800 seconds.",
        ))
    }
}

pub fn validate_alert_thresholds(values: &[u8]) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::validation(
            "At least one alert threshold is required.",
        ));
    }
    let mut previous = None;
    for value in values {
        if *value > 100 {
            return Err(AppError::validation(
                "Alert thresholds must be between 0 and 100.",
            ));
        }
        if previous.is_some_and(|prev| *value >= prev) {
            return Err(AppError::validation(
                "Alert thresholds must be unique and descending.",
            ));
        }
        previous = Some(*value);
    }
    Ok(())
}

pub fn validate_discord_webhook_url(value: &str) -> Result<(), AppError> {
    let parsed = Url::parse(value).map_err(|_| {
        AppError::new(
            ErrorCode::DiscordWebhookInvalid,
            "Discord webhook URL must be a valid HTTPS Discord webhook endpoint.",
            false,
        )
    })?;

    let host = parsed.host_str().unwrap_or_default();
    let valid_host = matches!(
        host,
        "discord.com" | "discordapp.com" | "canary.discord.com"
    );
    let valid_path = parsed.path().starts_with("/api/webhooks/");
    if parsed.scheme() == "https" && valid_host && valid_path {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::DiscordWebhookInvalid,
            "Discord webhook URL must be a valid HTTPS Discord webhook endpoint.",
            false,
        ))
    }
}
