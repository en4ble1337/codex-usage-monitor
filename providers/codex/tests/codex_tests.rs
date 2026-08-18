use async_trait::async_trait;
use chrono::Utc;
use ida_codex::*;
use ida_core::*;
use std::sync::{Arc, Mutex};

fn parse_fixture(name: &str) -> ProviderReadResult {
    let fixture = match name {
        "success.txt" => include_str!("../fixtures/success.txt"),
        "missing_weekly.txt" => include_str!("../fixtures/missing_weekly.txt"),
        "unauthenticated.txt" => include_str!("../fixtures/unauthenticated.txt"),
        "changed_format.txt" => include_str!("../fixtures/changed_format.txt"),
        "prototype_shape.txt" => include_str!("../fixtures/prototype_shape.txt"),
        other => panic!("unknown fixture {other}"),
    };
    parse_codex_status_result(
        fixture,
        CodexParseOptions {
            capture_method: CaptureMethod::Fixture,
            source_platform: SourcePlatform::Windows,
            scraped_at: Utc::now(),
        },
    )
}

#[test]
fn parser_extracts_successful_codex_output() {
    let result = parse_fixture("success.txt");

    assert_eq!(result.result_type, ProviderReadResultType::Success);
    let snapshot = result.snapshot.expect("snapshot");
    assert_eq!(snapshot.provider_id, "codex");
    assert_eq!(snapshot.provider_name, "Codex");
    assert_eq!(snapshot.capture_method, CaptureMethod::Fixture);
    assert_eq!(snapshot.limits.len(), 2);
    assert_eq!(snapshot.limits[0].id, "5h");
    assert_eq!(snapshot.limits[0].remaining_pct, 89);
    assert_eq!(snapshot.limits[0].used_pct, 11);
    assert_eq!(
        snapshot.limits[0].raw_reset_text.as_deref(),
        Some("in 2h 14m")
    );
    assert_eq!(snapshot.limits[1].id, "weekly");
    assert_eq!(snapshot.limits[1].remaining_pct, 64);
}

#[test]
fn parser_returns_partial_when_one_limit_is_missing() {
    let result = parse_fixture("missing_weekly.txt");

    assert_eq!(result.result_type, ProviderReadResultType::Partial);
    assert_eq!(result.snapshot.as_ref().expect("snapshot").limits.len(), 1);
    assert_eq!(
        result.error.as_ref().expect("partial error").code,
        ErrorCode::PartialSnapshot
    );
}

#[test]
fn parser_maps_authentication_and_changed_format_errors() {
    assert_eq!(
        parse_fixture("unauthenticated.txt")
            .error
            .expect("auth error")
            .code,
        ErrorCode::CodexUnauthenticated
    );
    assert_eq!(
        parse_fixture("changed_format.txt")
            .error
            .expect("parser error")
            .code,
        ErrorCode::ParserFailed
    );
}

#[test]
fn parser_preserves_useful_prototype_shape_compatibility() {
    let result = parse_fixture("prototype_shape.txt");
    let snapshot = result.snapshot.expect("snapshot");

    assert_eq!(snapshot.limits[0].remaining_pct, 42);
    assert_eq!(snapshot.limits[1].remaining_pct, 12);
    assert!(snapshot
        .limits
        .iter()
        .all(|limit| limit.remaining_pct + limit.used_pct == 100));
}

#[derive(Clone)]
struct ScriptedExecutor {
    outputs: Arc<Mutex<Vec<Result<CommandOutput, AppError>>>>,
    requests: Arc<Mutex<Vec<CommandRequest>>>,
}

impl ScriptedExecutor {
    fn new(outputs: Vec<Result<CommandOutput, AppError>>) -> Self {
        Self {
            outputs: Arc::new(Mutex::new(outputs)),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn requests(&self) -> Vec<CommandRequest> {
        self.requests.lock().expect("requests").clone()
    }
}

#[async_trait]
impl CommandExecutor for ScriptedExecutor {
    async fn run(&self, request: CommandRequest) -> Result<CommandOutput, AppError> {
        self.requests.lock().expect("requests").push(request);
        self.outputs.lock().expect("outputs").remove(0)
    }
}

fn success_output() -> CommandOutput {
    CommandOutput {
        stdout: include_str!("../fixtures/success.txt").to_string(),
        stderr: String::new(),
        status_success: true,
    }
}

#[tokio::test]
async fn provider_refresh_returns_native_success() {
    let executor = ScriptedExecutor::new(vec![Ok(success_output())]);
    let provider = CodexProvider::with_executor(executor.clone());
    let result = provider
        .refresh(&ProviderRuntimeConfig {
            capture_mode: CaptureMode::NativeOnly,
            timeout_seconds: 20,
            operation_id: "test".to_string(),
        })
        .await;

    assert_eq!(result.result_type, ProviderReadResultType::Success);
    assert_eq!(executor.requests()[0].program, "codex");
    assert_eq!(executor.requests()[0].args, vec!["/status"]);
}

#[tokio::test]
async fn provider_falls_back_to_wsl_when_native_codex_is_missing() {
    let executor = ScriptedExecutor::new(vec![
        Err(AppError::new(
            ErrorCode::CodexNotFound,
            "Codex CLI was not found.",
            true,
        )),
        Ok(success_output()),
    ]);
    let provider = CodexProvider::with_executor(executor.clone());
    let result = provider
        .refresh(&ProviderRuntimeConfig {
            capture_mode: CaptureMode::NativeThenWsl,
            timeout_seconds: 20,
            operation_id: "test".to_string(),
        })
        .await;

    assert_eq!(result.result_type, ProviderReadResultType::Success);
    assert_eq!(executor.requests()[0].program, "codex");
    assert_eq!(executor.requests()[1].program, "wsl.exe");
}

#[tokio::test]
async fn native_only_and_wsl_only_use_expected_capture_path() {
    let native_executor = ScriptedExecutor::new(vec![Ok(success_output())]);
    CodexProvider::with_executor(native_executor.clone())
        .refresh(&ProviderRuntimeConfig {
            capture_mode: CaptureMode::NativeOnly,
            timeout_seconds: 20,
            operation_id: "test".to_string(),
        })
        .await;
    assert_eq!(native_executor.requests().len(), 1);
    assert_eq!(native_executor.requests()[0].program, "codex");

    let wsl_executor = ScriptedExecutor::new(vec![Ok(success_output())]);
    CodexProvider::with_executor(wsl_executor.clone())
        .refresh(&ProviderRuntimeConfig {
            capture_mode: CaptureMode::WslOnly,
            timeout_seconds: 20,
            operation_id: "test".to_string(),
        })
        .await;
    assert_eq!(wsl_executor.requests().len(), 1);
    assert_eq!(wsl_executor.requests()[0].program, "wsl.exe");
}

#[tokio::test]
async fn provider_maps_wsl_missing_timeout_and_partial_results() {
    let missing = ScriptedExecutor::new(vec![Err(AppError::new(
        ErrorCode::WslNotFound,
        "WSL was not found.",
        true,
    ))]);
    let result = CodexProvider::with_executor(missing)
        .refresh(&ProviderRuntimeConfig {
            capture_mode: CaptureMode::WslOnly,
            timeout_seconds: 20,
            operation_id: "test".to_string(),
        })
        .await;
    assert_eq!(
        result.error.expect("wsl error").code,
        ErrorCode::WslNotFound
    );

    let timeout = ScriptedExecutor::new(vec![Err(AppError::new(
        ErrorCode::CaptureTimeout,
        "Codex status capture timed out.",
        true,
    ))]);
    let result = CodexProvider::with_executor(timeout)
        .refresh(&ProviderRuntimeConfig {
            capture_mode: CaptureMode::NativeOnly,
            timeout_seconds: 1,
            operation_id: "test".to_string(),
        })
        .await;
    assert_eq!(
        result.error.expect("timeout error").code,
        ErrorCode::CaptureTimeout
    );

    let partial = ScriptedExecutor::new(vec![Ok(CommandOutput {
        stdout: include_str!("../fixtures/missing_5h.txt").to_string(),
        stderr: String::new(),
        status_success: true,
    })]);
    let result = CodexProvider::with_executor(partial)
        .refresh(&ProviderRuntimeConfig {
            capture_mode: CaptureMode::NativeOnly,
            timeout_seconds: 20,
            operation_id: "test".to_string(),
        })
        .await;
    assert_eq!(result.result_type, ProviderReadResultType::Partial);
}
