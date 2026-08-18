use crate::{parse_codex_status_result, CodexParseOptions};
use async_trait::async_trait;
use chrono::Utc;
use ida_core::{
    AppError, CaptureMethod, CaptureMode, ErrorCode, ProviderReadResult, ProviderRuntimeConfig,
    SourcePlatform, UsageProvider,
};
use std::process::ExitStatus;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRequest {
    pub program: String,
    pub args: Vec<String>,
    pub timeout_seconds: u64,
    pub capture_method: CaptureMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status_success: bool,
}

#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn run(&self, request: CommandRequest) -> Result<CommandOutput, AppError>;
}

#[derive(Default)]
pub struct NativeCommandExecutor;

#[async_trait]
impl CommandExecutor for NativeCommandExecutor {
    async fn run(&self, request: CommandRequest) -> Result<CommandOutput, AppError> {
        let mut child = Command::new(&request.program);
        child.args(&request.args);
        child.kill_on_drop(true);
        child.stdout(std::process::Stdio::piped());
        child.stderr(std::process::Stdio::piped());

        let output = timeout(Duration::from_secs(request.timeout_seconds), child.output())
            .await
            .map_err(|_| {
                AppError::new(
                    ErrorCode::CaptureTimeout,
                    "Codex status capture timed out.",
                    true,
                )
            })?
            .map_err(|error| map_spawn_error(error, request.capture_method))?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            status_success: status_success(output.status),
        })
    }
}

pub struct CodexProvider<E: CommandExecutor = NativeCommandExecutor> {
    executor: Arc<E>,
}

impl CodexProvider<NativeCommandExecutor> {
    pub fn new() -> Self {
        Self {
            executor: Arc::new(NativeCommandExecutor),
        }
    }
}

impl Default for CodexProvider<NativeCommandExecutor> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: CommandExecutor> CodexProvider<E> {
    pub fn with_executor(executor: E) -> Self {
        Self {
            executor: Arc::new(executor),
        }
    }

    async fn capture_with_method(
        &self,
        method: CaptureMethod,
        config: &ProviderRuntimeConfig,
    ) -> Result<CommandOutput, AppError> {
        let request = match method {
            CaptureMethod::Native => CommandRequest {
                program: "codex".to_string(),
                args: vec!["/status".to_string()],
                timeout_seconds: config.timeout_seconds,
                capture_method: method,
            },
            CaptureMethod::Wsl => CommandRequest {
                program: "wsl.exe".to_string(),
                args: vec!["codex".to_string(), "/status".to_string()],
                timeout_seconds: config.timeout_seconds,
                capture_method: method,
            },
            CaptureMethod::Fixture | CaptureMethod::Unknown => {
                return Err(AppError::new(
                    ErrorCode::CaptureFailed,
                    "Unsupported Codex capture method.",
                    true,
                ));
            }
        };

        self.executor.run(request).await
    }
}

#[async_trait]
impl<E: CommandExecutor> UsageProvider for CodexProvider<E> {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    async fn refresh(&self, config: &ProviderRuntimeConfig) -> ProviderReadResult {
        let attempted_at = Utc::now();
        let capture_plan = match config.capture_mode {
            CaptureMode::NativeThenWsl => vec![CaptureMethod::Native, CaptureMethod::Wsl],
            CaptureMode::NativeOnly => vec![CaptureMethod::Native],
            CaptureMode::WslOnly => vec![CaptureMethod::Wsl],
        };

        let mut last_error = None;
        for method in capture_plan {
            match self.capture_with_method(method, config).await {
                Ok(output) if output.status_success => {
                    let raw = format!("{}\n{}", output.stdout, output.stderr);
                    return parse_codex_status_result(
                        &raw,
                        CodexParseOptions {
                            capture_method: method,
                            source_platform: SourcePlatform::current(),
                            scraped_at: attempted_at,
                        },
                    );
                }
                Ok(output) => {
                    let raw = format!("{}\n{}", output.stdout, output.stderr);
                    let parsed = parse_codex_status_result(
                        &raw,
                        CodexParseOptions {
                            capture_method: method,
                            source_platform: SourcePlatform::current(),
                            scraped_at: attempted_at,
                        },
                    );
                    if parsed.error.as_ref().is_some_and(|error| {
                        matches!(
                            error.code,
                            ErrorCode::CodexUnauthenticated | ErrorCode::ParserFailed
                        )
                    }) {
                        return parsed;
                    }
                    last_error = Some(AppError::new(
                        ErrorCode::CaptureFailed,
                        "Codex status command exited unsuccessfully.",
                        true,
                    ));
                }
                Err(error) => {
                    let should_try_wsl = config.capture_mode == CaptureMode::NativeThenWsl
                        && method == CaptureMethod::Native
                        && error.code == ErrorCode::CodexNotFound;
                    if should_try_wsl {
                        last_error = Some(error);
                        continue;
                    }
                    return ProviderReadResult::failure(attempted_at, Utc::now(), "codex", error);
                }
            }
        }

        ProviderReadResult::failure(
            attempted_at,
            Utc::now(),
            "codex",
            last_error.unwrap_or_else(|| {
                AppError::new(
                    ErrorCode::CaptureFailed,
                    "Codex status capture failed.",
                    true,
                )
            }),
        )
    }
}

fn map_spawn_error(error: std::io::Error, method: CaptureMethod) -> AppError {
    if error.kind() == std::io::ErrorKind::NotFound {
        match method {
            CaptureMethod::Native => AppError::new(
                ErrorCode::CodexNotFound,
                "Codex CLI was not found. Install Codex or update PATH, then refresh.",
                true,
            ),
            CaptureMethod::Wsl => AppError::new(
                ErrorCode::WslNotFound,
                "WSL was not found. Install WSL or choose native Codex capture.",
                true,
            ),
            _ => AppError::new(
                ErrorCode::CaptureFailed,
                "Capture command was not found.",
                true,
            ),
        }
    } else if method == CaptureMethod::Wsl {
        AppError::new(
            ErrorCode::WslUnavailable,
            "WSL could not run the Codex status command.",
            true,
        )
    } else {
        AppError::new(
            ErrorCode::CaptureFailed,
            "Codex status capture failed.",
            true,
        )
    }
}

fn status_success(status: ExitStatus) -> bool {
    status.success()
}
