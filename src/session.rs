use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const BUNDLE_ID: &str = "net.nilambar.tinytext";

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SessionState {
    pub(crate) workspace_root: Option<PathBuf>,
    pub(crate) tabs: Vec<PathBuf>,
    pub(crate) active_tab: Option<PathBuf>,
    pub(crate) expanded: Vec<PathBuf>,
    pub(crate) selected_path: Option<PathBuf>,
    pub(crate) sidebar_visible: bool,
}

pub(crate) fn session_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(BUNDLE_ID)
            .join("session.json"),
    )
}

pub(crate) fn load_session() -> Option<SessionState> {
    let path = session_path()?;
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_round_trips_through_json() {
        let state = SessionState {
            workspace_root: Some(PathBuf::from("/tmp/project")),
            tabs: vec![PathBuf::from("/tmp/project/a.rs")],
            active_tab: Some(PathBuf::from("/tmp/project/a.rs")),
            expanded: vec![PathBuf::from("/tmp/project")],
            selected_path: None,
            sidebar_visible: true,
        };
        let json = serde_json::to_string(&state).unwrap();
        let restored: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.workspace_root, state.workspace_root);
        assert_eq!(restored.tabs, state.tabs);
        assert_eq!(restored.active_tab, state.active_tab);
        assert_eq!(restored.expanded, state.expanded);
        assert_eq!(restored.selected_path, state.selected_path);
        assert!(restored.sidebar_visible);
    }

    #[test]
    fn session_defaults_missing_fields() {
        let restored: SessionState = serde_json::from_str(r#"{"tabs": ["/tmp/a.txt"]}"#).unwrap();

        assert_eq!(restored.tabs, [PathBuf::from("/tmp/a.txt")]);
        assert_eq!(restored.workspace_root, None);
        assert!(!restored.sidebar_visible);
    }
}
