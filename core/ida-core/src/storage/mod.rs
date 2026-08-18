use crate::{
    AppConfig, AppConfigPatch, AppDirs, AppError, AppState, ErrorCode, FreshnessStatus,
    ProviderReadResult, ProviderReadResultType, ProviderSnapshot, SecretUpdates, WidgetPreferences,
    WidgetPreferencesPatch,
};
use chrono::{DateTime, Duration, Utc};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SnapshotStore {
    latest_path: PathBuf,
    history_path: PathBuf,
}

impl SnapshotStore {
    pub fn new(dirs: &AppDirs) -> Self {
        Self {
            latest_path: dirs.latest_snapshot_path(),
            history_path: dirs.history_path(),
        }
    }

    pub fn read_latest(&self) -> Result<ProviderSnapshot, AppError> {
        read_json_file(
            &self.latest_path,
            ErrorCode::SnapshotNotFound,
            ErrorCode::SnapshotCorrupt,
        )
        .and_then(|snapshot: ProviderSnapshot| {
            snapshot
                .validate()
                .map_err(|error| with_code(error, ErrorCode::SnapshotCorrupt))?;
            Ok(snapshot)
        })
    }

    pub fn write_latest(&self, snapshot: &ProviderSnapshot) -> Result<(), AppError> {
        snapshot.validate()?;
        write_json_atomic(&self.latest_path, snapshot, ErrorCode::FileIoError)
    }

    pub fn append_history(
        &self,
        snapshot: &ProviderSnapshot,
        retention_hours: u64,
    ) -> Result<(), AppError> {
        snapshot.validate()?;
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::new(
                    ErrorCode::HistoryWriteFailed,
                    "Could not create history directory.",
                    true,
                )
                .with_detail("io", error.kind().to_string())
            })?;
        }

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.history_path)
            .map_err(|error| {
                AppError::new(
                    ErrorCode::HistoryWriteFailed,
                    "Could not open history file.",
                    true,
                )
                .with_detail("io", error.kind().to_string())
            })?;
        let line = serde_json::to_string(snapshot).map_err(|_| {
            AppError::new(
                ErrorCode::HistoryWriteFailed,
                "Could not encode history snapshot.",
                true,
            )
        })?;
        writeln!(file, "{line}").map_err(|error| {
            AppError::new(
                ErrorCode::HistoryWriteFailed,
                "Could not append history snapshot.",
                true,
            )
            .with_detail("io", error.kind().to_string())
        })?;
        self.trim_history(retention_hours)
    }

    pub fn apply_provider_result(
        &self,
        result: &ProviderReadResult,
        config: &AppConfig,
    ) -> Result<(), AppError> {
        if matches!(
            result.result_type,
            ProviderReadResultType::Success | ProviderReadResultType::Partial
        ) {
            if let Some(snapshot) = &result.snapshot {
                self.write_latest(snapshot)?;
                self.append_history(snapshot, config.history_retention_hours)?;
            }
        }
        Ok(())
    }

    pub fn build_app_state(
        &self,
        provider_id: &str,
        current_result: Option<&ProviderReadResult>,
        config: &AppConfig,
        now: DateTime<Utc>,
        next_poll_at: Option<DateTime<Utc>>,
    ) -> AppState {
        let latest_from_result = current_result.and_then(|result| result.snapshot.clone());
        let latest_from_disk = latest_from_result.or_else(|| self.read_latest().ok());
        assemble_app_state(
            provider_id,
            latest_from_disk,
            current_result,
            config,
            now,
            next_poll_at,
        )
    }

    fn trim_history(&self, retention_hours: u64) -> Result<(), AppError> {
        if !self.history_path.exists() {
            return Ok(());
        }
        let cutoff = Utc::now() - Duration::hours(retention_hours as i64);
        let file = fs::File::open(&self.history_path).map_err(|error| {
            AppError::new(
                ErrorCode::HistoryWriteFailed,
                "Could not read history for trimming.",
                true,
            )
            .with_detail("io", error.kind().to_string())
        })?;

        let mut retained = Vec::new();
        for line in std::io::BufReader::new(file).lines() {
            let line = line.map_err(|error| {
                AppError::new(
                    ErrorCode::HistoryWriteFailed,
                    "Could not read history line.",
                    true,
                )
                .with_detail("io", error.kind().to_string())
            })?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(snapshot) = serde_json::from_str::<ProviderSnapshot>(&line) {
                if snapshot.scraped_at >= cutoff {
                    retained.push(line);
                }
            }
        }

        let payload = if retained.is_empty() {
            String::new()
        } else {
            format!("{}\n", retained.join("\n"))
        };
        write_text_atomic(&self.history_path, &payload, ErrorCode::HistoryWriteFailed)
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(dirs: &AppDirs) -> Self {
        Self {
            path: dirs.config_path(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn read_or_create_default(&self) -> Result<AppConfig, AppError> {
        if !self.path.exists() {
            let config = AppConfig::default();
            self.write(&config)?;
            return Ok(config);
        }
        let config: AppConfig = read_json_file(
            &self.path,
            ErrorCode::ConfigReadFailed,
            ErrorCode::ConfigInvalid,
        )?;
        config
            .validate()
            .map_err(|error| with_code(error, ErrorCode::ConfigInvalid))?;
        Ok(config)
    }

    pub fn write(&self, config: &AppConfig) -> Result<(), AppError> {
        config.validate()?;
        write_json_atomic(&self.path, config, ErrorCode::ConfigWriteFailed)
    }

    pub fn patch(
        &self,
        patch: AppConfigPatch,
        secret_updates: Option<SecretUpdates>,
    ) -> Result<AppConfig, AppError> {
        let current = self.read_or_create_default()?;
        let next = current.apply_patch(patch, secret_updates)?;
        self.write(&next)?;
        Ok(next)
    }
}

#[derive(Debug, Clone)]
pub struct PreferencesStore {
    path: PathBuf,
}

impl PreferencesStore {
    pub fn new(dirs: &AppDirs) -> Self {
        Self {
            path: dirs.preferences_path(),
        }
    }

    pub fn read_or_create_default(&self) -> Result<WidgetPreferences, AppError> {
        if !self.path.exists() {
            let preferences = WidgetPreferences::default();
            self.write(&preferences)?;
            return Ok(preferences);
        }
        let preferences: WidgetPreferences = read_json_file(
            &self.path,
            ErrorCode::PreferencesInvalid,
            ErrorCode::PreferencesInvalid,
        )?;
        preferences
            .validate()
            .map_err(|error| with_code(error, ErrorCode::PreferencesInvalid))?;
        Ok(preferences)
    }

    pub fn write(&self, preferences: &WidgetPreferences) -> Result<(), AppError> {
        preferences.validate()?;
        write_json_atomic(&self.path, preferences, ErrorCode::PreferencesWriteFailed)
    }

    pub fn patch(&self, patch: WidgetPreferencesPatch) -> Result<WidgetPreferences, AppError> {
        let current = self.read_or_create_default()?;
        let next = current.apply_patch(patch)?;
        self.write(&next)?;
        Ok(next)
    }
}

pub fn assemble_app_state(
    provider_id: &str,
    latest_snapshot: Option<ProviderSnapshot>,
    current_result: Option<&ProviderReadResult>,
    config: &AppConfig,
    now: DateTime<Utc>,
    next_poll_at: Option<DateTime<Utc>>,
) -> AppState {
    let current_error = current_result.and_then(|result| result.error.clone());
    let last_attempted_at = current_result.map(|result| result.attempted_at);
    let last_success_at = latest_snapshot.as_ref().map(|snapshot| snapshot.scraped_at);

    let mut state = AppState {
        schema_version: crate::SCHEMA_VERSION,
        provider_id: provider_id.to_string(),
        latest_snapshot: latest_snapshot.clone(),
        current_error,
        freshness_status: FreshnessStatus::Unavailable,
        last_attempted_at,
        last_success_at,
        next_poll_at,
        effective_limits: Vec::new(),
    };

    let Some(snapshot) = latest_snapshot else {
        state.freshness_status = if current_result
            .and_then(|result| result.error.as_ref())
            .is_some()
        {
            FreshnessStatus::Error
        } else {
            FreshnessStatus::Unavailable
        };
        return state;
    };

    let age_seconds = now
        .signed_duration_since(snapshot.scraped_at)
        .num_seconds()
        .max(0) as u64;
    let current_failed =
        current_result.is_some_and(|result| result.result_type == ProviderReadResultType::Failure);
    let is_stale = age_seconds > config.stale_after_seconds || current_failed;

    state.freshness_status = if is_stale {
        FreshnessStatus::Stale
    } else {
        FreshnessStatus::Fresh
    };
    state.effective_limits = snapshot
        .limits
        .iter()
        .map(|limit| {
            if is_stale {
                limit.as_stale("Last known Codex data is stale.")
            } else {
                limit.clone()
            }
        })
        .collect();
    state
}

pub fn read_json_file<T: DeserializeOwned>(
    path: &Path,
    missing_code: ErrorCode,
    corrupt_code: ErrorCode,
) -> Result<T, AppError> {
    let payload = fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::new(
                missing_code,
                "Requested local file does not exist yet.",
                true,
            )
        } else {
            AppError::new(corrupt_code, "Could not read local JSON file.", true)
                .with_detail("io", error.kind().to_string())
        }
    })?;
    serde_json::from_str(&payload)
        .map_err(|_| AppError::new(corrupt_code, "Local JSON file is invalid.", true))
}

pub fn write_json_atomic<T: Serialize>(
    path: &Path,
    value: &T,
    code: ErrorCode,
) -> Result<(), AppError> {
    let payload = serde_json::to_string_pretty(value)
        .map_err(|_| AppError::new(code, "Could not encode JSON.", true))?;
    write_text_atomic(path, &payload, code)
}

pub fn write_text_atomic(path: &Path, payload: &str, code: ErrorCode) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::new(code, "Target path has no parent directory.", true))?;
    fs::create_dir_all(parent).map_err(|error| {
        AppError::new(code, "Could not create target directory.", true)
            .with_detail("io", error.kind().to_string())
    })?;

    let tmp_path = path.with_extension(format!(
        "{}tmp",
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}."))
            .unwrap_or_default()
    ));
    fs::write(&tmp_path, payload).map_err(|error| {
        AppError::new(code, "Could not write temporary file.", true)
            .with_detail("io", error.kind().to_string())
    })?;

    if path.exists() {
        fs::remove_file(path).map_err(|error| {
            AppError::new(code, "Could not replace existing file.", true)
                .with_detail("io", error.kind().to_string())
        })?;
    }
    fs::rename(&tmp_path, path).map_err(|error| {
        AppError::new(code, "Could not move temporary file into place.", true)
            .with_detail("io", error.kind().to_string())
    })
}

fn with_code(mut error: AppError, code: ErrorCode) -> AppError {
    error.code = code;
    error
}
