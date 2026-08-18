use crate::AppError;
use directories::BaseDirs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDirs {
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppDirs {
    pub fn resolve() -> Result<Self, AppError> {
        let base = BaseDirs::new()
            .ok_or_else(|| AppError::file_io("Could not resolve user directories."))?;

        #[cfg(target_os = "windows")]
        let dirs = {
            let config_dir = std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| base.config_dir().to_path_buf())
                .join("Ida");
            let local_data = std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| base.data_local_dir().to_path_buf())
                .join("Ida");
            Self {
                config_dir,
                state_dir: local_data.join("state"),
                logs_dir: local_data.join("logs"),
            }
        };

        #[cfg(target_os = "macos")]
        let dirs = {
            let config_dir = base.home_dir().join("Library/Application Support/Ida");
            Self {
                state_dir: config_dir.join("state"),
                logs_dir: base.home_dir().join("Library/Logs/Ida"),
                config_dir,
            }
        };

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        let dirs = {
            let config_dir = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| base.config_dir().to_path_buf())
                .join("ida");
            let state_dir = std::env::var_os("XDG_STATE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| base.data_local_dir().to_path_buf())
                .join("ida");
            Self {
                logs_dir: state_dir.join("logs"),
                config_dir,
                state_dir,
            }
        };

        Ok(dirs)
    }

    pub fn for_tests(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            config_dir: root.join("config"),
            state_dir: root.join("state"),
            logs_dir: root.join("logs"),
        }
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn preferences_path(&self) -> PathBuf {
        self.config_dir.join("widget-preferences.json")
    }

    pub fn latest_snapshot_path(&self) -> PathBuf {
        self.state_dir.join("latest.json")
    }

    pub fn history_path(&self) -> PathBuf {
        self.state_dir.join("history.ndjson")
    }

    pub fn alert_state_path(&self) -> PathBuf {
        self.state_dir.join("alert-state.json")
    }
}
