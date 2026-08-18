use ida_codex::CodexProvider;
use ida_core::{
    test_discord_webhook_with_transport, AppConfigPatch, AppConfigRedacted, AppDirs, AppError,
    AppRuntime, ConfigDiscordAlertChannel, ConfigStore, ErrorCode, PreferencesStore,
    ProviderRegistry, RefreshOutcome, RefreshReason, SecretPresence, SecretUpdates,
    WidgetPreferences, WidgetPreferencesPatch,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_notification::NotificationExt;

#[derive(Clone)]
pub struct DesktopState {
    runtime: Arc<AppRuntime>,
    dirs: AppDirs,
}

impl DesktopState {
    pub fn try_new(app: AppHandle) -> Result<Self, AppError> {
        let dirs = AppDirs::resolve()?;
        let registry = ProviderRegistry::new(vec![Arc::new(CodexProvider::new())]);
        let alert_store = ida_core::AlertStateStore::new(&dirs);
        let alert_manager = ida_core::AlertManager::new(alert_store)
            .with_channel(Arc::new(TauriNotificationChannel { app }))
            .with_channel(Arc::new(ConfigDiscordAlertChannel::new(ConfigStore::new(
                &dirs,
            ))));
        let runtime = Arc::new(AppRuntime::new(&dirs, registry, Some(alert_manager))?);
        Ok(Self { runtime, dirs })
    }
}

struct TauriNotificationChannel {
    app: AppHandle,
}

#[async_trait::async_trait]
impl ida_core::AlertChannel for TauriNotificationChannel {
    fn kind(&self) -> ida_core::AlertChannelKind {
        ida_core::AlertChannelKind::Native
    }

    async fn deliver(&self, notification: &ida_core::AlertNotification) -> Result<(), AppError> {
        self.app
            .notification()
            .builder()
            .title(notification.title())
            .body(notification.body())
            .show()
            .map_err(|_| {
                AppError::new(
                    ErrorCode::NotificationsUnavailable,
                    "Native notifications are unavailable.",
                    true,
                )
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GetAppStateResponse {
    pub state: ida_core::AppState,
    pub lowest_status: ida_core::LimitStatus,
    pub config_summary: Option<AppConfigRedacted>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GetConfigResponse {
    pub config: AppConfigRedacted,
    pub secret_presence: SecretPresence,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateConfigResponse {
    pub config: AppConfigRedacted,
    pub restart_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GetWidgetPreferencesResponse {
    pub preferences: WidgetPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UpdateWidgetPreferencesResponse {
    pub preferences: WidgetPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OpenDirectoryResponse {
    pub opened: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct QuitResponse {
    pub accepted: bool,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
pub struct RefreshUsageRequest {
    pub provider_id: Option<String>,
    pub reason: Option<RefreshReason>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
pub struct UpdateConfigRequest {
    pub patch: AppConfigPatch,
    pub secret_updates: Option<SecretUpdates>,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
pub struct UpdateWidgetPreferencesRequest {
    pub patch: WidgetPreferencesPatch,
}

#[derive(Debug, Clone, Deserialize, specta::Type)]
pub struct TestDiscordWebhookRequest {
    pub webhook_url: Option<String>,
}

#[tauri::command]
async fn get_app_state(
    state: State<'_, DesktopState>,
    include_config_summary: Option<bool>,
) -> Result<GetAppStateResponse, AppError> {
    let app_state = state.runtime.get_state().await?;
    let config_summary = if include_config_summary.unwrap_or(false) {
        let config = ConfigStore::new(&state.dirs).read_or_create_default()?;
        Some(config.redacted(Some(state.dirs.config_path().to_string_lossy().to_string())))
    } else {
        None
    };
    Ok(GetAppStateResponse {
        lowest_status: app_state.lowest_status(),
        state: app_state,
        config_summary,
    })
}

#[tauri::command]
async fn refresh_usage(
    app: AppHandle,
    state: State<'_, DesktopState>,
    request: Option<RefreshUsageRequest>,
) -> Result<RefreshOutcome, AppError> {
    let request = request.unwrap_or(RefreshUsageRequest {
        provider_id: None,
        reason: Some(RefreshReason::Manual),
    });
    let outcome = state
        .runtime
        .refresh_usage(
            request.provider_id,
            request.reason.unwrap_or(RefreshReason::Manual),
        )
        .await?;
    let _ = app.emit("ida:state-changed", &outcome.state);
    Ok(outcome)
}

#[tauri::command]
fn get_config(state: State<'_, DesktopState>) -> Result<GetConfigResponse, AppError> {
    let config = ConfigStore::new(&state.dirs).read_or_create_default()?;
    let path = state.dirs.config_path().to_string_lossy().to_string();
    Ok(GetConfigResponse {
        secret_presence: SecretPresence {
            discord_webhook_url: config.discord_webhook_url.is_some(),
        },
        config: config.redacted(Some(path.clone())),
        config_path: path,
    })
}

#[tauri::command]
async fn update_config(
    app: AppHandle,
    state: State<'_, DesktopState>,
    request: UpdateConfigRequest,
) -> Result<UpdateConfigResponse, AppError> {
    let config = state
        .runtime
        .update_config(request.patch, request.secret_updates)?;
    let app_state = state.runtime.get_state().await?;
    let _ = app.emit("ida:state-changed", &app_state);
    Ok(UpdateConfigResponse {
        config: config.redacted(Some(state.dirs.config_path().to_string_lossy().to_string())),
        restart_required: false,
    })
}

#[tauri::command]
fn get_widget_preferences(
    state: State<'_, DesktopState>,
) -> Result<GetWidgetPreferencesResponse, AppError> {
    Ok(GetWidgetPreferencesResponse {
        preferences: PreferencesStore::new(&state.dirs).read_or_create_default()?,
    })
}

#[tauri::command]
async fn update_widget_preferences(
    app: AppHandle,
    state: State<'_, DesktopState>,
    request: UpdateWidgetPreferencesRequest,
) -> Result<UpdateWidgetPreferencesResponse, AppError> {
    let preferences = state.runtime.update_widget_preferences(request.patch)?;
    apply_widget_preferences(&app, &preferences);
    let app_state = state.runtime.get_state().await?;
    let _ = app.emit("ida:state-changed", &app_state);
    Ok(UpdateWidgetPreferencesResponse { preferences })
}

#[tauri::command]
fn open_config_directory(
    state: State<'_, DesktopState>,
) -> Result<OpenDirectoryResponse, AppError> {
    std::fs::create_dir_all(&state.dirs.config_dir).map_err(|error| {
        AppError::new(
            ErrorCode::FileIoError,
            "Could not create Ida config directory.",
            true,
        )
        .with_detail("io", error.kind().to_string())
    })?;
    open_path(&state.dirs.config_dir)?;
    Ok(OpenDirectoryResponse {
        opened: true,
        path: state.dirs.config_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn test_discord_webhook(
    state: State<'_, DesktopState>,
    request: Option<TestDiscordWebhookRequest>,
) -> Result<ida_core::DiscordTestResult, AppError> {
    let config = ConfigStore::new(&state.dirs).read_or_create_default()?;
    let supplied = request.and_then(|request| request.webhook_url);
    test_discord_webhook_with_transport(
        config.discord_webhook_url.as_deref(),
        supplied.as_deref(),
        ida_core::ReqwestDiscordTransport::new(),
    )
    .await
}

#[tauri::command]
fn quit_app(app: AppHandle) -> Result<QuitResponse, AppError> {
    let app_for_exit = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        app_for_exit.exit(0);
    });
    Ok(QuitResponse { accepted: true })
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_or_create_widget(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            refresh_usage,
            get_config,
            update_config,
            get_widget_preferences,
            update_widget_preferences,
            open_config_directory,
            test_discord_webhook,
            quit_app
        ])
        .setup(|app| {
            let desktop_state = DesktopState::try_new(app.handle().clone())?;
            app.manage(desktop_state);
            ensure_windows(app.handle())?;
            install_widget_close_handler(app.handle());
            create_tray(app.handle())?;
            if let Ok(preferences) = app.state::<DesktopState>().runtime.get_widget_preferences() {
                apply_widget_preferences(app.handle(), &preferences);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Ida");
}

fn ensure_windows(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("widget").is_none() {
        WebviewWindowBuilder::new(app, "widget", WebviewUrl::App("widget.html".into()))
            .title("Ida")
            .inner_size(280.0, 160.0)
            .min_inner_size(280.0, 160.0)
            .decorations(false)
            .resizable(true)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(true)
            .build()?;
    }
    if app.get_webview_window("settings").is_none() {
        WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
            .title("Ida Settings")
            .inner_size(720.0, 640.0)
            .min_inner_size(520.0, 520.0)
            .visible(false)
            .build()?;
    }
    Ok(())
}

fn install_widget_close_handler(app: &AppHandle) {
    if let Some(widget) = app.get_webview_window("widget") {
        let app = app.clone();
        widget.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Some(window) = app.get_webview_window("widget") {
                    let _ = window.hide();
                }
                if let Some(state) = app.try_state::<DesktopState>() {
                    let _ = state
                        .runtime
                        .update_widget_preferences(WidgetPreferencesPatch {
                            visible_on_launch: Some(false),
                            ..WidgetPreferencesPatch::default()
                        });
                }
            }
        });
    }
}

fn apply_widget_preferences(app: &AppHandle, preferences: &WidgetPreferences) {
    if let Some(widget) = app.get_webview_window("widget") {
        let _ = widget.set_always_on_top(preferences.always_on_top);
        let _ = widget.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: preferences.width as f64,
            height: preferences.height as f64,
        }));
        if let (Some(x), Some(y)) = (preferences.position_x, preferences.position_y) {
            let _ = widget.set_position(tauri::Position::Logical(tauri::LogicalPosition {
                x: x as f64,
                y: y as f64,
            }));
        }
        if preferences.visible_on_launch {
            let _ = widget.show();
            let _ = widget.set_focus();
        } else {
            let _ = widget.hide();
        }
    }
}

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show-widget", "Show Widget", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide-widget", "Hide Widget", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh-now", "Refresh Now", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &refresh, &settings, &quit])?;
    let mut builder = TrayIconBuilder::with_id("ida-main")
        .tooltip("Ida")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show-widget" => show_or_create_widget(app),
            "hide-widget" => {
                if let Some(window) = app.get_webview_window("widget") {
                    let _ = window.hide();
                }
                if let Some(state) = app.try_state::<DesktopState>() {
                    let _ = state
                        .runtime
                        .update_widget_preferences(WidgetPreferencesPatch {
                            visible_on_launch: Some(false),
                            ..WidgetPreferencesPatch::default()
                        });
                }
            }
            "refresh-now" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app.try_state::<DesktopState>() {
                        if let Ok(outcome) = state
                            .runtime
                            .refresh_usage(None, RefreshReason::Manual)
                            .await
                        {
                            let _ = app.emit("ida:state-changed", outcome.state);
                        }
                    }
                });
            }
            "settings" => show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn show_or_create_widget(app: &AppHandle) {
    let _ = ensure_windows(app);
    if let Some(window) = app.get_webview_window("widget") {
        let _ = window.show();
        let _ = window.set_focus();
    }
    if let Some(state) = app.try_state::<DesktopState>() {
        let _ = state
            .runtime
            .update_widget_preferences(WidgetPreferencesPatch {
                visible_on_launch: Some(true),
                ..WidgetPreferencesPatch::default()
            });
    }
}

fn show_settings(app: &AppHandle) {
    let _ = ensure_windows(app);
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn open_path(path: &Path) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("explorer.exe");
        command.arg(path);
        command
    };
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = std::process::Command::new("open");
        command.arg(path);
        command
    };
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let mut command = {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(path);
        command
    };
    command.spawn().map_err(|error| {
        AppError::new(
            ErrorCode::FileIoError,
            "Could not open Ida config directory.",
            true,
        )
        .with_detail("io", error.kind().to_string())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDiscord;

    #[async_trait::async_trait]
    impl ida_core::DiscordTransport for MockDiscord {
        async fn post_json(
            &self,
            _webhook_url: &str,
            _payload: serde_json::Value,
        ) -> Result<u16, AppError> {
            Ok(204)
        }
    }

    #[tokio::test]
    async fn discord_test_uses_supplied_secret_without_exposing_it() {
        let result = test_discord_webhook_with_transport(
            None,
            Some("https://discord.com/api/webhooks/12345/test-token"),
            MockDiscord,
        )
        .await
        .expect("test webhook should send");

        assert_eq!(result.delivery_status, "sent");
        assert_eq!(result.status_code, Some(204));
    }
}
