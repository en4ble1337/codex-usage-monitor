use chrono::{DateTime, Utc};
use ida_core::{
    AppError, CaptureMethod, ErrorCode, LimitWindow, ProviderMetadata, ProviderReadResult,
    ProviderSnapshot, ProviderStatus, SourcePlatform,
};
use regex::Regex;

pub const CODEX_PARSER_VERSION: &str = "codex-status-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexParseOptions {
    pub capture_method: CaptureMethod,
    pub source_platform: SourcePlatform,
    pub scraped_at: DateTime<Utc>,
}

pub fn parse_codex_status_result(raw: &str, options: CodexParseOptions) -> ProviderReadResult {
    let attempted_at = options.scraped_at;
    match parse_codex_status(raw, &options) {
        Ok(ParseOutcome::Success(snapshot)) => {
            ProviderReadResult::success(attempted_at, Utc::now(), snapshot)
        }
        Ok(ParseOutcome::Partial(snapshot, error)) => {
            ProviderReadResult::partial(attempted_at, Utc::now(), snapshot, error)
        }
        Err(error) => ProviderReadResult::failure(attempted_at, Utc::now(), "codex", error),
    }
}

pub enum ParseOutcome {
    Success(ProviderSnapshot),
    Partial(ProviderSnapshot, AppError),
}

pub fn parse_codex_status(
    raw: &str,
    options: &CodexParseOptions,
) -> Result<ParseOutcome, AppError> {
    let cleaned = clean_terminal_output(raw);
    if cleaned.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::ParserFailed,
            "Codex status output was empty.",
            true,
        ));
    }

    if looks_unauthenticated(&cleaned) {
        return Err(AppError::new(
            ErrorCode::CodexUnauthenticated,
            "Codex is installed but not signed in. Open Codex once and sign in.",
            true,
        ));
    }

    let five_h = extract_limit(&cleaned, "5h", "5-hour")?;
    let weekly = extract_limit(&cleaned, "weekly", "Weekly")?;

    if five_h.is_none() && weekly.is_none() {
        return Err(AppError::new(
            ErrorCode::ParserFailed,
            "Could not parse Codex usage percentages.",
            true,
        ));
    }

    let mut limits = Vec::new();
    if let Some(limit) = five_h {
        limits.push(limit);
    }
    if let Some(limit) = weekly {
        limits.push(limit);
    }

    let mut metadata = ProviderMetadata::new(CODEX_PARSER_VERSION);
    metadata.account_label = extract_account_label(&cleaned);
    metadata.raw_model_label = extract_model_label(&cleaned);
    metadata
        .raw_fields
        .insert("source".to_string(), "codex_status".to_string());

    let provider_status = if limits.len() == 2 {
        ProviderStatus::Ok
    } else {
        ProviderStatus::Partial
    };
    let snapshot = ProviderSnapshot::new_codex(
        provider_status,
        options.scraped_at,
        options.capture_method,
        options.source_platform,
        limits,
        metadata,
    )?;

    if provider_status == ProviderStatus::Partial {
        let missing = if snapshot.limits.iter().any(|limit| limit.id == "5h") {
            "weekly"
        } else {
            "5h"
        };
        Ok(ParseOutcome::Partial(
            snapshot,
            AppError::new(
                ErrorCode::PartialSnapshot,
                "Codex status output was missing one expected usage window.",
                true,
            )
            .with_detail("missing_limit", missing),
        ))
    } else {
        Ok(ParseOutcome::Success(snapshot))
    }
}

pub fn clean_terminal_output(raw: &str) -> String {
    let ansi = Regex::new(r"\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\)|[@-_])")
        .expect("valid ansi regex");
    ansi.replace_all(raw, "")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('│', "|")
        .chars()
        .filter(|ch| *ch == '\n' || *ch == '\t' || !ch.is_control())
        .collect()
}

fn extract_limit(
    cleaned: &str,
    id: &'static str,
    label: &'static str,
) -> Result<Option<LimitWindow>, AppError> {
    let label_pattern = if id == "5h" {
        r"(?i)(5\s*[- ]?\s*h(?:our)?|5h)\s+limit"
    } else {
        r"(?i)weekly\s+limit"
    };
    let line_regex =
        Regex::new(&format!(r"(?m)^.*{label_pattern}.*$")).expect("valid limit line regex");
    let percent_regex =
        Regex::new(r"(?i)(\d{1,3})\s*%\s*(?:left|remaining)").expect("valid percent regex");
    let reset_regex = Regex::new(r"(?i)resets?\s+([^)|\n]+)").expect("valid reset regex");

    for line_match in line_regex.find_iter(cleaned) {
        let line = line_match.as_str();
        if let Some(percent_caps) = percent_regex.captures(line) {
            let remaining_pct: u8 = percent_caps
                .get(1)
                .and_then(|value| value.as_str().parse().ok())
                .ok_or_else(|| AppError::parser_failed("Could not parse Codex percentage."))?;
            let raw_reset_text = reset_regex
                .captures(line)
                .and_then(|captures| captures.get(1))
                .map(|value| value.as_str().trim().trim_end_matches('.').to_string())
                .filter(|value| !value.is_empty());
            let limit = LimitWindow::new(id, label, id, remaining_pct, raw_reset_text)?;
            return Ok(Some(limit));
        }
    }

    Ok(None)
}

fn extract_account_label(cleaned: &str) -> Option<String> {
    let regex = Regex::new(r"(?im)^\s*Account:\s*([^|\n]+)").expect("valid account regex");
    regex
        .captures(cleaned)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().trim().to_string())
        .filter(|value| !value.is_empty() && !ida_core::contains_sensitive_value(value))
}

fn extract_model_label(cleaned: &str) -> Option<String> {
    let regex = Regex::new(r"(?im)^\s*(?:Model|Plan):\s*([^|\n]+)").expect("valid model regex");
    regex
        .captures(cleaned)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().trim().to_string())
        .filter(|value| !value.is_empty() && !ida_core::contains_sensitive_value(value))
}

fn looks_unauthenticated(cleaned: &str) -> bool {
    let lower = cleaned.to_ascii_lowercase();
    [
        "not authenticated",
        "not signed in",
        "sign in",
        "log in",
        "login required",
        "authentication required",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}
