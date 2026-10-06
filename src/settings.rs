use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::paths::support_dir;

pub(crate) const MIN_FONT_SIZE: f32 = 6.;
pub(crate) const MAX_FONT_SIZE: f32 = 72.;
pub(crate) const MIN_FONT_WEIGHT: f32 = 100.;
pub(crate) const MAX_FONT_WEIGHT: f32 = 900.;
pub(crate) const DEFAULT_FONT_WEIGHT: f32 = 400.;

/// Suggested when the configured family has no close match; only installed
/// ones are shown.
const MONOSPACE_FONTS: &[&str] = &[
    "SF Mono",
    "Menlo",
    "Monaco",
    "JetBrains Mono",
    "Fira Code",
    "Source Code Pro",
    "Cascadia Code",
    "IBM Plex Mono",
    "Hack",
    "Courier New",
];
const MAX_SUGGESTIONS: usize = 3;

/// User-authored preferences. Grouped by area so new keys land in an existing
/// section (`editor.tab_size`) or a new one (`ui.font_size`) without renames.
///
/// Unknown keys are rejected so a typo like `font_sise` surfaces as an error
/// instead of silently doing nothing.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Settings {
    pub(crate) editor: EditorSettings,
}

/// `None` keeps the theme's monospace default.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct EditorSettings {
    pub(crate) font_family: Option<String>,
    pub(crate) font_size: Option<f32>,
    pub(crate) font_weight: Option<f32>,
}

impl EditorSettings {
    pub(crate) fn font_size(&self) -> Option<f32> {
        self.font_size
            .map(|size| size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE))
    }

    pub(crate) fn font_weight(&self) -> Option<f32> {
        self.font_weight
            .map(|weight| weight.clamp(MIN_FONT_WEIGHT, MAX_FONT_WEIGHT))
    }
}

pub(crate) fn settings_path() -> Option<PathBuf> {
    Some(support_dir()?.join("settings.json"))
}

pub(crate) fn parse_settings(data: &str) -> Result<Settings, serde_json::Error> {
    serde_json::from_str(data)
}

/// A missing file yields defaults. Unreadable or invalid files return a
/// message for the user; callers keep defaults or the previous settings.
pub(crate) fn load_settings() -> Result<Settings, String> {
    let Some(path) = settings_path() else {
        return Ok(Settings::default());
    };
    let data = match std::fs::read_to_string(&path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Settings::default());
        }
        Err(error) => return Err(format!("Could not read settings: {error}")),
    };
    parse_settings(&data).map_err(|error| format!("Invalid settings: {error}"))
}

/// Explains why `family` will not render, with installed names to use instead.
pub(crate) fn font_family_issue(family: &str, installed: &[String]) -> Option<String> {
    if installed.iter().any(|name| name == family) {
        return None;
    }

    let wanted = family.to_lowercase();
    let close: Vec<&str> = installed
        .iter()
        .filter(|name| {
            let name = name.to_lowercase();
            name.contains(&wanted) || wanted.contains(&name)
        })
        .take(MAX_SUGGESTIONS)
        .map(String::as_str)
        .collect();
    if !close.is_empty() {
        return Some(format!(
            "Font \"{family}\" is not installed. Did you mean {}?",
            quoted(&close)
        ));
    }

    let monospace: Vec<&str> = MONOSPACE_FONTS
        .iter()
        .copied()
        .filter(|font| installed.iter().any(|name| name == font))
        .collect();
    if monospace.is_empty() {
        return Some(format!("Font \"{family}\" is not installed."));
    }
    Some(format!(
        "Font \"{family}\" is not installed. Installed monospace fonts: {}.",
        quoted(&monospace)
    ))
}

fn quoted(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_editor_section() {
        let settings = parse_settings(
            r#"{"editor": {"font_family": "Menlo", "font_size": 15, "font_weight": 600}}"#,
        )
        .unwrap();

        assert_eq!(settings.editor.font_family.as_deref(), Some("Menlo"));
        assert_eq!(settings.editor.font_size(), Some(15.));
        assert_eq!(settings.editor.font_weight(), Some(600.));
    }

    #[test]
    fn defaults_missing_sections() {
        assert_eq!(parse_settings("{}").unwrap(), Settings::default());
        assert_eq!(
            parse_settings(r#"{"editor": {}}"#).unwrap(),
            Settings::default()
        );
    }

    #[test]
    fn rejects_unknown_keys() {
        let error = parse_settings(r#"{"editor": {"font_sise": 14}}"#).unwrap_err();
        assert!(error.to_string().contains("unknown field `font_sise`"));

        let error = parse_settings(r#"{"edtor": {}}"#).unwrap_err();
        assert!(error.to_string().contains("unknown field `edtor`"));
    }

    #[test]
    fn rejects_wrong_types() {
        assert!(parse_settings(r#"{"editor": {"font_size": "14"}}"#).is_err());
    }

    #[test]
    fn clamps_font_size() {
        let settings = parse_settings(r#"{"editor": {"font_size": 500}}"#).unwrap();
        assert_eq!(settings.editor.font_size(), Some(MAX_FONT_SIZE));

        let settings = parse_settings(r#"{"editor": {"font_size": 0}}"#).unwrap();
        assert_eq!(settings.editor.font_size(), Some(MIN_FONT_SIZE));
    }

    #[test]
    fn clamps_font_weight() {
        let settings = parse_settings(r#"{"editor": {"font_weight": 1000}}"#).unwrap();
        assert_eq!(settings.editor.font_weight(), Some(MAX_FONT_WEIGHT));

        let settings = parse_settings(r#"{"editor": {"font_weight": 50}}"#).unwrap();
        assert_eq!(settings.editor.font_weight(), Some(MIN_FONT_WEIGHT));
    }

    fn installed(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn accepts_installed_font() {
        assert_eq!(font_family_issue("Menlo", &installed(&["Menlo"])), None);
    }

    #[test]
    fn suggests_close_font_names() {
        let fonts = installed(&["JetBrains Mono", "JetBrains Mono NL", "Menlo"]);

        assert_eq!(
            font_family_issue("jetbrains mono", &fonts).unwrap(),
            r#"Font "jetbrains mono" is not installed. Did you mean "JetBrains Mono", "JetBrains Mono NL"?"#
        );
        assert_eq!(
            font_family_issue("JetBrains", &fonts).unwrap(),
            r#"Font "JetBrains" is not installed. Did you mean "JetBrains Mono", "JetBrains Mono NL"?"#
        );
    }

    #[test]
    fn lists_installed_monospace_fonts_without_close_match() {
        let fonts = installed(&["Arial", "Menlo", "Monaco"]);

        assert_eq!(
            font_family_issue("Comic Code", &fonts).unwrap(),
            r#"Font "Comic Code" is not installed. Installed monospace fonts: "Menlo", "Monaco"."#
        );
        assert_eq!(
            font_family_issue("Comic Code", &installed(&["Arial"])).unwrap(),
            r#"Font "Comic Code" is not installed."#
        );
    }

    #[test]
    fn rejects_malformed_json() {
        let missing_comma =
            "{\n  \"editor\": {\n    \"font_family\": \"Menlo\"\n    \"font_size\": 14\n  }\n}";
        let error = parse_settings(missing_comma).unwrap_err();
        assert_eq!(error.line(), 4);
    }
}
