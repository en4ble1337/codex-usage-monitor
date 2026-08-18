use async_trait::async_trait;
use chrono::{Duration, Utc};
use ida_core::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn codex_snapshot(remaining_5h: u8, remaining_weekly: u8) -> ProviderSnapshot {
    ProviderSnapshot::new_codex(
        ProviderStatus::Ok,
        Utc::now(),
        CaptureMethod::Fixture,
        SourcePlatform::Windows,
        vec![
            LimitWindow::new(
                "5h",
                "5-hour",
                "5h",
                remaining_5h,
                Some("in 2h".to_string()),
            )
            .expect("valid 5h limit"),
            LimitWindow::new(
                "weekly",
                "Weekly",
                "weekly",
                remaining_weekly,
                Some("Friday".to_string()),
            )
            .expect("valid weekly limit"),
        ],
        ProviderMetadata::new("test"),
    )
    .expect("valid snapshot")
}

#[test]
fn config_defaults_match_arch_contract() {
    let config = AppConfig::default();

    assert_eq!(config.polling_interval_seconds, 900);
    assert_eq!(config.stale_after_seconds, 1800);
    assert_eq!(config.alert_thresholds, vec![75, 50, 25, 10, 5]);
    assert_eq!(config.capture_mode, CaptureMode::NativeThenWsl);
    assert_eq!(config.history_retention_hours, 24);
    config.validate().expect("default config should validate");
}

#[test]
fn validation_rejects_bad_percentages_and_secret_metadata() {
    let limit = LimitWindow {
        remaining_pct: 101,
        used_pct: 0,
        ..LimitWindow::new("5h", "5-hour", "5h", 50, None).expect("base limit")
    };
    assert_eq!(
        limit.validate().expect_err("invalid percentage").code,
        ErrorCode::ValidationError
    );

    let mut snapshot = codex_snapshot(80, 70);
    snapshot.metadata.raw_fields.insert(
        "leak".to_string(),
        "https://discord.com/api/webhooks/123/secret".to_string(),
    );
    assert_eq!(
        snapshot.validate().expect_err("secret metadata").code,
        ErrorCode::ValidationError
    );
}

#[test]
fn redacted_config_never_exposes_discord_webhook_url() {
    let config = AppConfig {
        discord_webhook_url: Some("https://discord.com/api/webhooks/123/secret".to_string()),
        ..AppConfig::default()
    };

    let redacted = config.redacted(Some("safe/path/config.json".to_string()));
    assert!(redacted.discord_webhook_configured);
    assert_eq!(
        redacted.discord_webhook_masked.as_deref(),
        Some("https://discord.com/api/webhooks/...")
    );
    let serialized = serde_json::to_string(&redacted).expect("serializable");
    assert!(!serialized.contains("secret"));
}

#[test]
fn app_error_serializes_frontend_safe_shape() {
    let error = AppError::new(
        ErrorCode::CodexNotFound,
        "Codex CLI was not found. Install Codex or update PATH, then refresh.",
        true,
    );
    let value = serde_json::to_value(error).expect("serializable");

    assert_eq!(value["code"], "CODEX_NOT_FOUND");
    assert!(value["operation_id"]
        .as_str()
        .is_some_and(|id| id.len() >= 32));
    assert_eq!(value["retryable"], true);
}

#[test]
fn config_store_patches_and_clears_secret() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dirs = AppDirs::for_tests(temp.path());
    let store = ConfigStore::new(&dirs);

    let updated = store
        .patch(
            AppConfigPatch {
                polling_interval_seconds: Some(120),
                ..AppConfigPatch::default()
            },
            Some(SecretUpdates {
                discord_webhook_url: Some(
                    "https://discord.com/api/webhooks/123/secret".to_string(),
                ),
            }),
        )
        .expect("patch should save");
    assert_eq!(updated.polling_interval_seconds, 120);
    assert!(updated.discord_webhook_url.is_some());

    let cleared = store
        .patch(
            AppConfigPatch::default(),
            Some(SecretUpdates {
                discord_webhook_url: Some(String::new()),
            }),
        )
        .expect("clear should save");
    assert!(cleared.discord_webhook_url.is_none());
}

#[test]
fn invalid_config_and_preferences_return_structured_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dirs = AppDirs::for_tests(temp.path());
    std::fs::create_dir_all(&dirs.config_dir).expect("config dir");
    std::fs::write(dirs.config_path(), "{ invalid").expect("write malformed config");
    std::fs::write(dirs.preferences_path(), "{ invalid").expect("write malformed prefs");

    assert_eq!(
        ConfigStore::new(&dirs)
            .read_or_create_default()
            .expect_err("malformed config")
            .code,
        ErrorCode::ConfigInvalid
    );
    assert_eq!(
        PreferencesStore::new(&dirs)
            .read_or_create_default()
            .expect_err("malformed preferences")
            .code,
        ErrorCode::PreferencesInvalid
    );
}

#[test]
fn snapshot_store_preserves_latest_on_failed_refresh_and_marks_stale() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dirs = AppDirs::for_tests(temp.path());
    let store = SnapshotStore::new(&dirs);
    let config = AppConfig::default();
    let snapshot = codex_snapshot(80, 70);
    store.write_latest(&snapshot).expect("latest write");

    let result = ProviderReadResult::failure(
        Utc::now(),
        Utc::now(),
        "codex",
        AppError::new(
            ErrorCode::ParserFailed,
            "Could not parse Codex usage.",
            true,
        ),
    );
    store
        .apply_provider_result(&result, &config)
        .expect("failed result should not overwrite");

    let latest = store.read_latest().expect("latest still exists");
    assert_eq!(latest.limits[0].remaining_pct, 80);

    let state = store.build_app_state("codex", Some(&result), &config, Utc::now(), None);
    assert_eq!(state.freshness_status, FreshnessStatus::Stale);
    assert!(state
        .effective_limits
        .iter()
        .all(|limit| limit.status == LimitStatus::Stale));
}

#[test]
fn corrupt_latest_snapshot_returns_snapshot_corrupt() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dirs = AppDirs::for_tests(temp.path());
    std::fs::create_dir_all(&dirs.state_dir).expect("state dir");
    std::fs::write(dirs.latest_snapshot_path(), "{ invalid").expect("write corrupt");

    assert_eq!(
        SnapshotStore::new(&dirs)
            .read_latest()
            .expect_err("corrupt latest")
            .code,
        ErrorCode::SnapshotCorrupt
    );
}

struct MockProvider {
    result: ProviderReadResult,
}

#[async_trait]
impl UsageProvider for MockProvider {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    async fn refresh(&self, _config: &ProviderRuntimeConfig) -> ProviderReadResult {
        self.result.clone()
    }
}

#[tokio::test]
async fn runtime_uses_provider_trait_and_updates_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dirs = AppDirs::for_tests(temp.path());
    let snapshot = codex_snapshot(88, 66);
    let result = ProviderReadResult::success(Utc::now(), Utc::now(), snapshot);
    let runtime = AppRuntime::new(
        &dirs,
        ProviderRegistry::new(vec![Arc::new(MockProvider { result })]),
        None,
    )
    .expect("runtime");

    let outcome = runtime
        .refresh_usage(None, RefreshReason::Manual)
        .await
        .expect("refresh");

    assert_eq!(outcome.state.freshness_status, FreshnessStatus::Fresh);
    assert_eq!(outcome.state.effective_limits.len(), 2);
}

#[tokio::test]
async fn unknown_provider_returns_provider_not_found() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dirs = AppDirs::for_tests(temp.path());
    let runtime = AppRuntime::new(&dirs, ProviderRegistry::new(Vec::new()), None).expect("runtime");

    assert_eq!(
        runtime
            .refresh_usage(Some("missing".to_string()), RefreshReason::Manual)
            .await
            .expect_err("unknown provider")
            .code,
        ErrorCode::ProviderNotFound
    );
}

struct CountingAlertChannel {
    count: Arc<AtomicUsize>,
}

#[async_trait]
impl AlertChannel for CountingAlertChannel {
    fn kind(&self) -> AlertChannelKind {
        AlertChannelKind::Native
    }

    async fn deliver(&self, _notification: &AlertNotification) -> Result<(), AppError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn alert_manager_fires_once_per_threshold_and_reset_window() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dirs = AppDirs::for_tests(temp.path());
    let count = Arc::new(AtomicUsize::new(0));
    let manager = AlertManager::new(AlertStateStore::new(&dirs)).with_channel(Arc::new(
        CountingAlertChannel {
            count: Arc::clone(&count),
        },
    ));
    let previous = codex_snapshot(80, 80);
    let current = codex_snapshot(49, 80);

    let first = manager
        .evaluate(Some(&previous), &current, &AppConfig::default())
        .await
        .expect("first evaluate");
    let duplicate = manager
        .evaluate(Some(&previous), &current, &AppConfig::default())
        .await
        .expect("duplicate evaluate");

    assert_eq!(first.len(), 2);
    assert!(duplicate.is_empty());
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn discord_webhook_validation_and_test_transport_work() {
    struct MockTransport;

    #[async_trait]
    impl DiscordTransport for MockTransport {
        async fn post_json(
            &self,
            _webhook_url: &str,
            _payload: serde_json::Value,
        ) -> Result<u16, AppError> {
            Ok(204)
        }
    }

    assert_eq!(
        validate_discord_webhook_url("http://example.com/hook")
            .expect_err("invalid webhook")
            .code,
        ErrorCode::DiscordWebhookInvalid
    );

    let result = test_discord_webhook_with_transport(
        None,
        Some("https://discord.com/api/webhooks/123/test"),
        MockTransport,
    )
    .await
    .expect("mock delivery");
    assert_eq!(result.status_code, Some(204));
}

#[test]
fn stale_detection_uses_utc_age() {
    let snapshot = ProviderSnapshot {
        scraped_at: Utc::now() - Duration::seconds(1801),
        ..codex_snapshot(80, 80)
    };
    let state = assemble_app_state(
        "codex",
        Some(snapshot),
        None,
        &AppConfig::default(),
        Utc::now(),
        None,
    );
    assert_eq!(state.freshness_status, FreshnessStatus::Stale);
}
