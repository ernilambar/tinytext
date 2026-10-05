mod files;
mod tabs;
mod ui;

use std::collections::HashSet;
use std::path::PathBuf;

use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _, input::EditorState, notification::Notification,
};
use gpui_kit::*;

use crate::cli::{app_bundle_path, install_cli};
use crate::session::{SessionState, session_path};
use crate::{About, InstallCli, Quit, ToggleSidebar};

struct OpenTab {
    path: Option<PathBuf>,
    title: SharedString,
    language: SharedString,
    dirty: bool,
    editor: Entity<EditorState>,
    _subscriptions: Vec<Subscription>,
}

pub(crate) struct TinytextApp {
    pub(crate) focus_handle: FocusHandle,
    workspace_root: Option<PathBuf>,
    expanded: HashSet<PathBuf>,
    selected_path: Option<PathBuf>,
    tabs: Vec<OpenTab>,
    active_tab: Option<usize>,
    context_tab: Option<usize>,
    sidebar_visible: bool,
    cursor_line: usize,
    cursor_col: usize,
}

impl TinytextApp {
    pub(crate) fn new(workspace_root: Option<PathBuf>, cx: &mut Context<Self>) -> Self {
        let mut expanded = HashSet::new();
        if let Some(root) = &workspace_root {
            expanded.insert(root.clone());
        }
        let sidebar_visible = workspace_root.is_some();

        Self {
            focus_handle: cx.focus_handle(),
            workspace_root,
            expanded,
            selected_path: None,
            tabs: Vec::new(),
            active_tab: None,
            context_tab: None,
            sidebar_visible,
            cursor_line: 1,
            cursor_col: 1,
        }
    }

    pub(crate) fn restore_session(
        &mut self,
        session: SessionState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(root) = session.workspace_root.filter(|path| path.is_dir()) {
            self.workspace_root = Some(root);
        }

        self.expanded = session
            .expanded
            .into_iter()
            .filter(|path| path.is_dir())
            .collect();
        if let Some(root) = &self.workspace_root {
            self.expanded.insert(root.clone());
        }

        self.selected_path = session.selected_path.filter(|path| path.exists());
        self.sidebar_visible = session.sidebar_visible && self.workspace_root.is_some();

        let mut paths = session.tabs;
        paths.retain(|path| path.is_file());
        let active = session.active_tab.filter(|path| path.is_file());

        cx.spawn_in(window, async move |this, cx| {
            for path in paths {
                let read = cx
                    .background_executor()
                    .spawn({
                        let path = path.clone();
                        async move { std::fs::read_to_string(path) }
                    })
                    .await;
                if let Ok(content) = read {
                    this.update_in(cx, |this, window, cx| {
                        this.add_tab(Some(path), content, window, cx);
                    })
                    .ok();
                }
            }

            this.update_in(cx, |this, window, cx| {
                if let Some(active) = active
                    && let Some(ix) = this
                        .tabs
                        .iter()
                        .position(|tab| tab.path.as_deref() == Some(active.as_path()))
                {
                    this.activate_tab(ix, window, cx);
                    this.save_session();
                }
            })
            .ok();
        })
        .detach();

        cx.notify();
    }

    fn save_session(&self) {
        let Some(path) = session_path() else {
            return;
        };

        let mut expanded: Vec<PathBuf> = self.expanded.iter().cloned().collect();
        expanded.sort();

        let state = SessionState {
            workspace_root: self.workspace_root.clone(),
            tabs: self
                .tabs
                .iter()
                .filter_map(|tab| tab.path.clone())
                .collect(),
            active_tab: self.active().and_then(|tab| tab.path.clone()),
            expanded,
            selected_path: self.selected_path.clone(),
            sidebar_visible: self.sidebar_visible,
        };

        let Ok(json) = serde_json::to_string_pretty(&state) else {
            return;
        };
        if let Some(parent) = path.parent()
            && std::fs::create_dir_all(parent).is_err()
        {
            return;
        }

        let temp = path.with_file_name("session.json.tmp");
        if std::fs::write(&temp, json).is_ok() {
            let _ = std::fs::rename(&temp, &path);
        }
    }

    fn active(&self) -> Option<&OpenTab> {
        self.active_tab.and_then(|ix| self.tabs.get(ix))
    }

    fn active_language(&self) -> SharedString {
        self.active()
            .map(|tab| tab.language.clone())
            .unwrap_or_else(|| "Plain Text".into())
    }

    fn active_dirty(&self) -> bool {
        self.active().map(|tab| tab.dirty).unwrap_or(false)
    }

    fn on_quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn on_toggle_sidebar(
        &mut self,
        _: &ToggleSidebar,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_sidebar(cx);
    }

    fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        if self.workspace_root.is_none() {
            return;
        }
        self.sidebar_visible = !self.sidebar_visible;
        self.save_session();
        cx.notify();
    }

    fn on_install_cli(&mut self, _: &InstallCli, window: &mut Window, cx: &mut Context<Self>) {
        let Some(bundle) = app_bundle_path() else {
            window.push_notification(
                Notification::error(
                    "Run the bundled Tinytext.app to install the command-line tool".to_string(),
                ),
                cx,
            );
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { install_cli(&bundle) })
                .await;

            this.update_in(cx, |_this, window, cx| match result {
                Ok(path) => window.push_notification(
                    Notification::success(format!("Installed command at {}", path.display())),
                    cx,
                ),
                Err(message) => window.push_notification(
                    Notification::error(format!("Could not install command: {message}")),
                    cx,
                ),
            })
            .ok();
        })
        .detach();
    }

    fn on_about(&mut self, _: &About, window: &mut Window, cx: &mut Context<Self>) {
        window.open_alert_dialog(cx, |alert, _, _| {
            alert
                .title("Tinytext")
                .description(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .child("A native macOS text editor built with GPUI Kit.")
                        .child(env!("CARGO_PKG_REPOSITORY"))
                        .child(env!("CARGO_PKG_HOMEPAGE")),
                )
                .ok_text("Close")
        });
    }
}

impl Render for TinytextApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("tinytext-root")
            .v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_sm()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_new_file))
            .on_action(cx.listener(Self::on_open_file))
            .on_action(cx.listener(Self::on_open_folder))
            .on_action(cx.listener(Self::on_save_file))
            .on_action(cx.listener(Self::on_close_tab))
            .on_action(cx.listener(Self::on_quit))
            .on_action(cx.listener(Self::on_toggle_sidebar))
            .on_action(cx.listener(Self::on_install_cli))
            .on_action(cx.listener(Self::on_about))
            .child(self.render_menu_bar(cx))
            .child(self.render_tab_bar(cx))
            .child(self.render_workspace(cx))
            .child(self.render_status_bar(cx))
    }
}
