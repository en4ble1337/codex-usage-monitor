use crate::{
    validate_alert_thresholds, validate_discord_webhook_url, AppConfig, AppDirs, AppError,
    ConfigStore, ErrorCode, LimitWindow, ProviderSnapshot,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AlertChannelKind {
    Native,
    Discord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Sent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AlertStateEntry {
    pub provider_id: String,
    pub limit_id: String,
    pub threshold: u8,
    pub channel: AlertChannelKind,
    pub reset_window_key: String,
    pub sent_at: DateTime<Utc>,
    pub delivery_status: DeliveryStatus,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct AlertState {
    pub schema_version: u16,
    pub entries: Vec<AlertStateEntry>,
    pub updated_at: DateTime<Utc>,
}

impl AlertState {
    pub fn empty() -> Self {
        Self {
            schema_version: crate::SCHEMA_VERSION,
            entries: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    pub fn has_entry(
        &self,
        provider_id: &str,
        limit_id: &str,
        threshold: u8,
        channel: AlertChannelKind,
        reset_window_key: &str,
    ) -> bool {
        self.entries.iter().any(|entry| {
            entry.provider_id == provider_id
                && entry.limit_id == limit_id
                && entry.threshold == threshold
                && entry.channel == channel
                && entry.reset_window_key == reset_window_key
        })
    }

    pub fn record(&mut self, entry: AlertStateEntry) {
        self.entries.push(entry);
        self.updated_at = Utc::now();
    }

    pub fn prune_to_snapshot(&mut self, snapshot: &ProviderSnapshot) {
        self.entries.retain(|entry| {
            snapshot.limits.iter().any(|limit| {
                limit.id == entry.limit_id && reset_window_key(limit) == entry.reset_window_key
            })
        });
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone)]
pub struct AlertStateStore {
    path: PathBuf,
}

impl AlertStateStore {
    pub fn new(dirs: &AppDirs) -> Self {
        Self {
            path: dirs.alert_state_path(),
        }
    }

    pub fn read_or_default(&self) -> Result<AlertState, AppError> {
        if !self.path.exists() {
            return Ok(AlertState::empty());
        }
        crate::read_json_file(
            &self.path,
            ErrorCode::SnapshotNotFound,
            ErrorCode::AlertStateWriteFailed,
        )
        .or_else(|_| Ok(AlertState::empty()))
    }

    pub fn write(&self, state: &AlertState) -> Result<(), AppError> {
        crate::write_json_atomic(&self.path, state, ErrorCode::AlertStateWriteFailed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertNotification {
    pub provider_id: String,
    pub provider_name: String,
    pub limit_id: String,
    pub limit_label: String,
    pub remaining_pct: u8,
    pub threshold: u8,
    pub reset_text: Option<String>,
}

impl AlertNotification {
    pub fn title(&self) -> String {
        format!(
            "{} {} at {}% remaining",
            self.provider_name, self.limit_label, self.remaining_pct
        )
    }

    pub fn body(&self) -> String {
        let reset = self
            .reset_text
            .as_ref()
            .map(|value| format!(" Resets {value}."))
            .unwrap_or_default();
        format!(
            "{} crossed the {}% alert threshold.{}",
            self.limit_label, self.threshold, reset
        )
    }
}

#[async_trait]
pub trait AlertChannel: Send + Sync {
    fn kind(&self) -> AlertChannelKind;
    async fn deliver(&self, notification: &AlertNotification) -> Result<(), AppError>;
}

#[derive(Default)]
pub struct NoopNativeNotificationChannel;

#[async_trait]
impl AlertChannel for NoopNativeNotificationChannel {
    fn kind(&self) -> AlertChannelKind {
        AlertChannelKind::Native
    }

    async fn deliver(&self, _notification: &AlertNotification) -> Result<(), AppError> {
        Ok(())
    }
}

#[async_trait]
pub trait DiscordTransport: Send + Sync {
    async fn post_json(
        &self,
        webhook_url: &str,
        payload: serde_json::Value,
    ) -> Result<u16, AppError>;
}

#[derive(Default)]
pub struct ReqwestDiscordTransport {
    client: reqwest::Client,
}

impl ReqwestDiscordTransport {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }
}

#[async_trait]
impl DiscordTransport for ReqwestDiscordTransport {
    async fn post_json(
        &self,
        webhook_url: &str,
        payload: serde_json::Value,
    ) -> Result<u16, AppError> {
        validate_discord_webhook_url(webhook_url)?;
        let response = self
            .client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|_| {
                AppError::new(
                    ErrorCode::DiscordDeliveryFailed,
                    "Discord webhook delivery failed.",
                    true,
                )
            })?;
        let status = response.status().as_u16();
        if response.status().is_success() {
            Ok(status)
        } else {
            Err(AppError::new(
                ErrorCode::DiscordDeliveryFailed,
                "Discord webhook returned a failure status.",
                true,
            )
            .with_detail("status", status.to_string()))
        }
    }
}

pub struct DiscordAlertChannel<T: DiscordTransport = ReqwestDiscordTransport> {
    webhook_url: String,
    transport: T,
}

impl DiscordAlertChannel<ReqwestDiscordTransport> {
    pub fn from_webhook_url(webhook_url: impl Into<String>) -> Result<Self, AppError> {
        let webhook_url = webhook_url.into();
        validate_discord_webhook_url(&webhook_url)?;
        Ok(Self {
            webhook_url,
            transport: ReqwestDiscordTransport::new(),
        })
    }
}

impl<T: DiscordTransport> DiscordAlertChannel<T> {
    pub fn with_transport(webhook_url: impl Into<String>, transport: T) -> Result<Self, AppError> {
        let webhook_url = webhook_url.into();
        validate_discord_webhook_url(&webhook_url)?;
        Ok(Self {
            webhook_url,
            transport,
        })
    }

    pub async fn send_test(&self) -> Result<DiscordTestResult, AppError> {
        let payload = serde_json::json!({
            "content": "Ida test alert: Discord delivery is configured."
        });
        let status_code = self.transport.post_json(&self.webhook_url, payload).await?;
        Ok(DiscordTestResult {
            delivery_status: "sent".to_string(),
            status_code: Some(status_code),
            message: "Discord test alert sent.".to_string(),
        })
    }
}

#[async_trait]
impl<T: DiscordTransport> AlertChannel for DiscordAlertChannel<T> {
    fn kind(&self) -> AlertChannelKind {
        AlertChannelKind::Discord
    }

    async fn deliver(&self, notification: &AlertNotification) -> Result<(), AppError> {
        let payload = serde_json::json!({
            "content": format!(
                "{} {} is at {}% remaining (threshold {}%).{}",
                notification.provider_name,
                notification.limit_label,
                notification.remaining_pct,
                notification.threshold,
                notification
                    .reset_text
                    .as_ref()
                    .map(|reset| format!(" Resets {reset}."))
                    .unwrap_or_default()
            )
        });
        self.transport.post_json(&self.webhook_url, payload).await?;
        Ok(())
    }
}

pub struct ConfigDiscordAlertChannel<T: DiscordTransport = ReqwestDiscordTransport> {
    config_store: ConfigStore,
    transport: T,
}

impl ConfigDiscordAlertChannel<ReqwestDiscordTransport> {
    pub fn new(config_store: ConfigStore) -> Self {
        Self {
            config_store,
            transport: ReqwestDiscordTransport::new(),
        }
    }
}

impl<T: DiscordTransport> ConfigDiscordAlertChannel<T> {
    pub fn with_transport(config_store: ConfigStore, transport: T) -> Self {
        Self {
            config_store,
            transport,
        }
    }
}

#[async_trait]
impl<T: DiscordTransport> AlertChannel for ConfigDiscordAlertChannel<T> {
    fn kind(&self) -> AlertChannelKind {
        AlertChannelKind::Discord
    }

    async fn deliver(&self, notification: &AlertNotification) -> Result<(), AppError> {
        let config = self.config_store.read_or_create_default()?;
        let webhook_url = config.discord_webhook_url.ok_or_else(|| {
            AppError::new(
                ErrorCode::DiscordNotConfigured,
                "No Discord webhook is configured.",
                false,
            )
        })?;
        let payload = serde_json::json!({
            "content": format!(
                "{} {} is at {}% remaining (threshold {}%).{}",
                notification.provider_name,
                notification.limit_label,
                notification.remaining_pct,
                notification.threshold,
                notification
                    .reset_text
                    .as_ref()
                    .map(|reset| format!(" Resets {reset}."))
                    .unwrap_or_default()
            )
        });
        self.transport.post_json(&webhook_url, payload).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DiscordTestResult {
    pub delivery_status: String,
    pub status_code: Option<u16>,
    pub message: String,
}

pub struct AlertManager {
    store: AlertStateStore,
    channels: Vec<Arc<dyn AlertChannel>>,
}

impl AlertManager {
    pub fn new(store: AlertStateStore) -> Self {
        Self {
            store,
            channels: Vec::new(),
        }
    }

    pub fn with_channel(mut self, channel: Arc<dyn AlertChannel>) -> Self {
        self.channels.push(channel);
        self
    }

    pub async fn evaluate(
        &self,
        previous: Option<&ProviderSnapshot>,
        current: &ProviderSnapshot,
        config: &AppConfig,
    ) -> Result<Vec<AlertStateEntry>, AppError> {
        validate_alert_thresholds(&config.alert_thresholds)?;
        let mut state = self.store.read_or_default()?;
        state.prune_to_snapshot(current);
        let mut recorded = Vec::new();

        for channel in &self.channels {
            if channel.kind() == AlertChannelKind::Native && !config.native_notifications_enabled {
                continue;
            }
            if channel.kind() == AlertChannelKind::Discord && !config.discord_alerts_enabled {
                continue;
            }

            for limit in &current.limits {
                let Some(previous_remaining) =
                    previous.and_then(|snapshot| find_limit(snapshot, &limit.id))
                else {
                    continue;
                };

                for threshold in &config.alert_thresholds {
                    if previous_remaining > *threshold && limit.remaining_pct <= *threshold {
                        let reset_key = reset_window_key(limit);
                        if state.has_entry(
                            &current.provider_id,
                            &limit.id,
                            *threshold,
                            channel.kind(),
                            &reset_key,
                        ) {
                            continue;
                        }

                        let notification = AlertNotification {
                            provider_id: current.provider_id.clone(),
                            provider_name: current.provider_name.clone(),
                            limit_id: limit.id.clone(),
                            limit_label: limit.label.clone(),
                            remaining_pct: limit.remaining_pct,
                            threshold: *threshold,
                            reset_text: limit
                                .raw_reset_text
                                .clone()
                                .or_else(|| limit.resets_at.map(|value| value.to_rfc3339())),
                        };
                        let result = channel.deliver(&notification).await;
                        let entry = AlertStateEntry {
                            provider_id: current.provider_id.clone(),
                            limit_id: limit.id.clone(),
                            threshold: *threshold,
                            channel: channel.kind(),
                            reset_window_key: reset_key,
                            sent_at: Utc::now(),
                            delivery_status: if result.is_ok() {
                                DeliveryStatus::Sent
                            } else {
                                DeliveryStatus::Failed
                            },
                            error_code: result.err().map(|error| error.code.as_str().to_string()),
                        };
                        state.record(entry.clone());
                        recorded.push(entry);
                    }
                }
            }
        }

        self.store.write(&state)?;
        Ok(recorded)
    }
}

pub async fn test_discord_webhook_with_transport<T: DiscordTransport>(
    configured_webhook_url: Option<&str>,
    supplied_webhook_url: Option<&str>,
    transport: T,
) -> Result<DiscordTestResult, AppError> {
    let webhook_url = supplied_webhook_url
        .or(configured_webhook_url)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::DiscordNotConfigured,
                "No Discord webhook is configured.",
                false,
            )
        })?;
    let channel = DiscordAlertChannel::with_transport(webhook_url, transport)?;
    channel.send_test().await
}

fn find_limit(snapshot: &ProviderSnapshot, limit_id: &str) -> Option<u8> {
    snapshot
        .limits
        .iter()
        .find(|limit| limit.id == limit_id)
        .map(|limit| limit.remaining_pct)
}

pub fn reset_window_key(limit: &LimitWindow) -> String {
    if let Some(resets_at) = limit.resets_at {
        return resets_at.to_rfc3339();
    }
    let raw = limit
        .raw_reset_text
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let mut hasher = DefaultHasher::new();
    raw.hash(&mut hasher);
    format!("raw:{:016x}", hasher.finish())
}
