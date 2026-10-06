mod files;
mod tabs;
mod ui;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _, input::EditorState, link::Link, notification::Notification,
};
use gpui_kit::*;

use crate::cli::{app_bundle_path, install_cli};
use crate::session::{SessionState, session_path};
use crate::settings::{
    DEFAULT_FONT_WEIGHT, EditorSettings, Settings, font_family_issue, load_settings, settings_path,
};
use crate::update::{INSTALL_COMMAND, is_newer, latest_version};
use crate::{About, CheckForUpdates, InstallCli, OpenSettings, Quit, ToggleSidebar};

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
    settings: Settings,
}

impl TinytextApp {
    pub(crate) fn new(
        workspace_root: Option<PathBuf>,
        settings: Settings,
        cx: &mut Context<Self>,
    ) -> Self {
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
            settings,
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

    fn on_check_for_updates(
        &mut self,
        _: &CheckForUpdates,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async { latest_version() })
                .await;

            this.update_in(cx, |_this, window, cx| {
                let current = env!("CARGO_PKG_VERSION");
                match result {
                    Ok(latest) if is_newer(&latest, current) => {
                        window.open_alert_dialog(cx, move |alert, _, _| {
                            alert
                                .title("Update Available")
                                .description(
                                    div()
                                        .v_flex()
                                        .gap_1()
                                        .child(format!(
                                            "Tinytext {latest} is available. You have {current}."
                                        ))
                                        .child("Save your work, then run this in Terminal:")
                                        .child(INSTALL_COMMAND),
                                )
                                .ok_text("Copy Command")
                                .show_cancel(true)
                                .cancel_text("Later")
                                .on_ok(|_, window, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        INSTALL_COMMAND.to_string(),
                                    ));
                                    window.push_notification(
                                        Notification::success("Copied install command"),
                                        cx,
                                    );
                                    true
                                })
                        });
                    }
                    Ok(_) => window.push_notification(
                        Notification::success(format!("Tinytext {current} is up to date")),
                        cx,
                    ),
                    Err(message) => window.push_notification(
                        Notification::error(format!("Could not check for updates: {message}")),
                        cx,
                    ),
                }
            })
            .ok();
        })
        .detach();
    }

    fn on_open_settings(&mut self, _: &OpenSettings, window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = settings_path() else {
            return;
        };

        if !path.exists() {
            let starter = Settings {
                editor: EditorSettings {
                    font_family: Some(cx.theme().mono_font_family.to_string()),
                    font_size: Some(cx.theme().mono_font_size.as_f32()),
                    font_weight: Some(DEFAULT_FONT_WEIGHT),
                },
            };
            let written = serde_json::to_string_pretty(&starter)
                .map_err(std::io::Error::other)
                .and_then(|json| {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, json + "\n")
                });
            if let Err(error) = written {
                window.push_notification(
                    Notification::error(format!("Could not create settings: {error}")),
                    cx,
                );
                return;
            }
        }

        self.request_open(path, window, cx);
    }

    /// Re-applies settings when the saved file is the settings file. Invalid
    /// JSON keeps the previous settings and reports the parse error.
    pub(super) fn reload_settings_if(
        &mut self,
        path: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if settings_path().as_deref() != Some(path) {
            return;
        }

        match load_settings() {
            Ok(settings) => {
                self.settings = settings;
                let base_weight = self
                    .settings
                    .editor
                    .font_weight()
                    .unwrap_or(DEFAULT_FONT_WEIGHT);
                crate::markdown::apply_emphasis(cx, base_weight);
                self.check_font_family(window, cx);
                cx.notify();
            }
            Err(message) => window.push_notification(Notification::error(message), cx),
        }
    }

    /// Warns when the configured editor font is not installed. Font enumeration
    /// is slow on macOS, so it runs off the main thread.
    pub(crate) fn check_font_family(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(family) = self.settings.editor.font_family.clone() else {
            return;
        };
        let text_system = cx.text_system().clone();

        cx.spawn_in(window, async move |this, cx| {
            let installed = cx
                .background_executor()
                .spawn(async move { text_system.all_font_names() })
                .await;
            if let Some(message) = font_family_issue(&family, &installed) {
                this.update_in(cx, |_this, window, cx| {
                    window.push_notification(Notification::warning(message), cx);
                })
                .ok();
            }
        })
        .detach();
    }

    fn on_about(&mut self, _: &About, window: &mut Window, cx: &mut Context<Self>) {
        window.open_alert_dialog(cx, |alert, _, cx| {
            alert
                .description(
                    div()
                        .v_flex()
                        .w_full()
                        .items_center()
                        .gap_2()
                        .child(about_icon())
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(cx.theme().foreground)
                                .child("Tinytext"),
                        )
                        .child(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .child("A native macOS text editor built with GPUI Kit.")
                        .child(
                            div()
                                .v_flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    Link::new("about-repository")
                                        .href(env!("CARGO_PKG_REPOSITORY"))
                                        .child(env!("CARGO_PKG_REPOSITORY")),
                                )
                                .child(
                                    Link::new("about-homepage")
                                        .href(env!("CARGO_PKG_HOMEPAGE"))
                                        .child(env!("CARGO_PKG_HOMEPAGE")),
                                ),
                        ),
                )
                .ok_text("Close")
        });
    }
}

fn about_icon() -> impl IntoElement {
    img(Arc::new(Image::from_bytes(
        ImageFormat::Png,
        include_bytes!("../../assets/icon.png").to_vec(),
    )))
    .w(px(96.))
    .h(px(96.))
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
            .on_action(cx.listener(Self::on_check_for_updates))
            .on_action(cx.listener(Self::on_about))
            .on_action(cx.listener(Self::on_open_settings))
            .child(self.render_workspace(cx))
            .child(self.render_status_bar(cx))
    }
}
