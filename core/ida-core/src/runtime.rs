use crate::{
    AlertManager, AppConfig, AppDirs, AppError, AppState, ConfigStore, ErrorCode, PreferencesStore,
    ProviderReadResult, ProviderRuntimeConfig, SecretUpdates, SnapshotStore, UsageProvider,
    WidgetPreferences, WidgetPreferencesPatch,
};
use chrono::{Duration, Utc};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

#[derive(
    Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "snake_case")]
pub enum RefreshReason {
    Startup,
    #[default]
    Manual,
    Poll,
    Test,
}

#[derive(Clone)]
pub struct ProviderRegistry {
    providers: Arc<BTreeMap<String, Arc<dyn UsageProvider>>>,
}

impl ProviderRegistry {
    pub fn new(providers: Vec<Arc<dyn UsageProvider>>) -> Self {
        let providers = providers
            .into_iter()
            .map(|provider| (provider.id().to_string(), provider))
            .collect();
        Self {
            providers: Arc::new(providers),
        }
    }

    pub fn get(&self, provider_id: &str) -> Option<Arc<dyn UsageProvider>> {
        self.providers.get(provider_id).cloned()
    }
}

pub struct AppRuntime {
    config_store: ConfigStore,
    preferences_store: PreferencesStore,
    snapshot_store: SnapshotStore,
    provider_registry: ProviderRegistry,
    alert_manager: Option<AlertManager>,
    state: RwLock<AppState>,
    refresh_lock: Mutex<()>,
}

impl AppRuntime {
    pub fn new(
        dirs: &AppDirs,
        provider_registry: ProviderRegistry,
        alert_manager: Option<AlertManager>,
    ) -> Result<Self, AppError> {
        let config_store = ConfigStore::new(dirs);
        let preferences_store = PreferencesStore::new(dirs);
        let snapshot_store = SnapshotStore::new(dirs);
        let config = config_store.read_or_create_default()?;
        let initial_state = snapshot_store.build_app_state(
            &config.active_provider_id,
            None,
            &config,
            Utc::now(),
            None,
        );
        Ok(Self {
            config_store,
            preferences_store,
            snapshot_store,
            provider_registry,
            alert_manager,
            state: RwLock::new(initial_state),
            refresh_lock: Mutex::new(()),
        })
    }

    pub async fn get_state(&self) -> Result<AppState, AppError> {
        Ok(self.state.read().await.clone())
    }

    pub fn config_store(&self) -> &ConfigStore {
        &self.config_store
    }

    pub fn preferences_store(&self) -> &PreferencesStore {
        &self.preferences_store
    }

    pub async fn startup_refresh(&self) -> Result<RefreshOutcome, AppError> {
        self.refresh_usage(None, RefreshReason::Startup).await
    }

    pub async fn refresh_usage(
        &self,
        provider_id: Option<String>,
        reason: RefreshReason,
    ) -> Result<RefreshOutcome, AppError> {
        let _guard = self.refresh_lock.lock().await;
        let config = self.config_store.read_or_create_default()?;
        let provider_id = provider_id.unwrap_or_else(|| config.active_provider_id.clone());
        let provider = self.provider_registry.get(&provider_id).ok_or_else(|| {
            AppError::new(
                ErrorCode::ProviderNotFound,
                "Requested usage provider is not registered.",
                false,
            )
            .with_detail("provider_id", provider_id.clone())
        })?;

        let previous = self.snapshot_store.read_latest().ok();
        let operation_id = Uuid::new_v4().to_string();
        let runtime_config = ProviderRuntimeConfig::from_app_config(&config, operation_id);
        let result = provider.refresh(&runtime_config).await;
        self.snapshot_store
            .apply_provider_result(&result, &config)?;

        if let (Some(alert_manager), Some(snapshot)) = (&self.alert_manager, &result.snapshot) {
            let _ = alert_manager
                .evaluate(previous.as_ref(), snapshot, &config)
                .await;
        }

        let next_poll_at = match reason {
            RefreshReason::Poll | RefreshReason::Startup | RefreshReason::Manual => {
                Some(Utc::now() + Duration::seconds(config.polling_interval_seconds as i64))
            }
            RefreshReason::Test => None,
        };
        let state = self.snapshot_store.build_app_state(
            &provider_id,
            Some(&result),
            &config,
            Utc::now(),
            next_poll_at,
        );
        *self.state.write().await = state.clone();
        Ok(RefreshOutcome { result, state })
    }

    pub async fn scheduled_tick(&self) -> Result<RefreshOutcome, AppError> {
        self.refresh_usage(None, RefreshReason::Poll).await
    }

    pub fn update_config(
        &self,
        patch: crate::AppConfigPatch,
        secret_updates: Option<SecretUpdates>,
    ) -> Result<AppConfig, AppError> {
        self.config_store.patch(patch, secret_updates)
    }

    pub fn get_widget_preferences(&self) -> Result<WidgetPreferences, AppError> {
        self.preferences_store.read_or_create_default()
    }

    pub fn update_widget_preferences(
        &self,
        patch: WidgetPreferencesPatch,
    ) -> Result<WidgetPreferences, AppError> {
        self.preferences_store.patch(patch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct RefreshOutcome {
    pub result: ProviderReadResult,
    pub state: AppState,
}
