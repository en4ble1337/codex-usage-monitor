use crate::{AppConfig, CaptureMode, ProviderReadResult};
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRuntimeConfig {
    pub capture_mode: CaptureMode,
    pub timeout_seconds: u64,
    pub operation_id: String,
}

impl ProviderRuntimeConfig {
    pub fn from_app_config(config: &AppConfig, operation_id: impl Into<String>) -> Self {
        Self {
            capture_mode: config.capture_mode,
            timeout_seconds: 20,
            operation_id: operation_id.into(),
        }
    }
}

#[async_trait]
pub trait UsageProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    async fn refresh(&self, config: &ProviderRuntimeConfig) -> ProviderReadResult;
}
