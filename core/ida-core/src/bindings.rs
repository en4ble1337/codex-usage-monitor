pub fn exported_type_names() -> Vec<&'static str> {
    vec![
        "AppConfig",
        "AppConfigPatch",
        "AppConfigRedacted",
        "AppState",
        "ProviderSnapshot",
        "ProviderReadResult",
        "LimitWindow",
        "WidgetPreferences",
        "WidgetPreferencesPatch",
        "AlertState",
        "AlertStateEntry",
        "AppError",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_registry_mentions_app_state() {
        assert!(exported_type_names().contains(&"AppState"));
    }
}
