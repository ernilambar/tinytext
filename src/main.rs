use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, Size, Theme, ThemeMode, WindowExt as _,
    badge::Badge,
    button::{Button, ButtonVariant, ButtonVariants as _},
    input::{Editor, EditorState, InputEvent},
    list::ListItem,
    menu::{DropdownMenu as _, PopupMenuItem},
    notification::Notification,
    resizable::{h_resizable, resizable_panel},
    status_bar::StatusBar,
    tab::{Tab, TabBar},
};
use gpui_kit::*;

gpui_kit::actions!(
    tinytext,
    [
        NewFile,
        OpenFile,
        OpenFolder,
        SaveFile,
        CloseTab,
        Quit,
        ToggleSidebar,
        EditUndo,
        EditRedo,
        EditCut,
        EditCopy,
        EditPaste,
        EditSelectAll,
    ]
);

const SIDEBAR_WIDTH: Pixels = px(240.);
const LANGUAGES: [&str; 11] = [
    "Plain Text",
    "Rust",
    "TOML",
    "JSON",
    "Markdown",
    "JavaScript",
    "TypeScript",
    "Python",
    "HTML",
    "CSS",
    "PHP",
];

struct OpenTab {
    path: Option<PathBuf>,
    title: SharedString,
    language: SharedString,
    dirty: bool,
    editor: Entity<EditorState>,
    _subscriptions: Vec<Subscription>,
}

struct TinytextApp {
    focus_handle: FocusHandle,
    workspace_root: Option<PathBuf>,
    expanded: HashSet<PathBuf>,
    selected_path: Option<PathBuf>,
    tabs: Vec<OpenTab>,
    active_tab: Option<usize>,
    sidebar_visible: bool,
    cursor_line: usize,
    cursor_col: usize,
}

impl TinytextApp {
    fn new(workspace_root: Option<PathBuf>, cx: &mut Context<Self>) -> Self {
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
            sidebar_visible,
            cursor_line: 1,
            cursor_col: 1,
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

    fn add_tab(
        &mut self,
        path: Option<PathBuf>,
        content: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let language: SharedString = path
            .as_deref()
            .map(language_for)
            .unwrap_or_else(|| "Plain Text".into());
        let title: SharedString = path
            .as_deref()
            .map(|path| file_name(path).into())
            .unwrap_or_else(|| "Untitled".into());

        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language(editor_language_id(&language))
                .default_value(content)
        });

        let change_subscription = cx.subscribe(&editor, |this, editor, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.set_dirty(editor.entity_id(), true, cx);
            }
        });
        let cursor_subscription = cx.observe(&editor, |this, editor, cx| {
            this.sync_cursor(editor.entity_id(), cx);
        });

        self.tabs.push(OpenTab {
            path,
            title,
            language,
            dirty: false,
            editor: editor.clone(),
            _subscriptions: vec![change_subscription, cursor_subscription],
        });
        self.active_tab = Some(self.tabs.len() - 1);

        editor.update(cx, |state, cx| state.focus(window, cx));
        let position = editor.read(cx).cursor_position();
        self.cursor_line = position.line as usize + 1;
        self.cursor_col = position.character as usize + 1;
        cx.notify();
    }

    fn request_open(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ix) = self
            .tabs
            .iter()
            .position(|tab| tab.path.as_deref() == Some(path.as_path()))
        {
            self.activate_tab(ix, window, cx);
            return;
        }

        cx.spawn_in(window, async move |this, cx| {
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
        })
        .detach();
    }

    fn on_new_file(&mut self, _: &NewFile, window: &mut Window, cx: &mut Context<Self>) {
        self.add_tab(None, String::new(), window, cx);
    }

    fn on_open_file(&mut self, _: &OpenFile, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Open File".into()),
        });

        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await {
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
            }
        })
        .detach();
    }

    fn on_open_folder(&mut self, _: &OpenFolder, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open Folder".into()),
        });

        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await
                && let Some(path) = paths.into_iter().next()
            {
                this.update_in(cx, |this, _window, cx| this.open_folder(path, cx))
                    .ok();
            }
        })
        .detach();
    }

    fn open_folder(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.workspace_root = Some(path.clone());
        self.expanded.clear();
        self.expanded.insert(path);
        self.selected_path = None;
        self.sidebar_visible = true;
        cx.notify();
    }

    fn on_save_file(&mut self, _: &SaveFile, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ix) = self.active_tab else {
            return;
        };
        let Some(tab) = self.tabs.get(ix) else {
            return;
        };
        let id = tab.editor.entity_id();

        if let Some(path) = tab.path.clone() {
            self.save_editor(id, path, window, cx);
            return;
        }

        let directory = self
            .workspace_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let receiver = cx.prompt_for_new_path(&directory, Some("untitled.txt"));

        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(path))) = receiver.await {
                this.update_in(cx, |this, window, cx| {
                    this.save_editor(id, path, window, cx)
                })
                .ok();
            }
        })
        .detach();
    }

    fn save_editor(
        &mut self,
        id: EntityId,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(tab) = self.tabs.iter().find(|tab| tab.editor.entity_id() == id) else {
            return;
        };
        let text = tab.editor.read(cx).text().to_string();
        let filename = file_name(&path);

        cx.spawn_in(window, async move |this, cx| {
            let write_path = path.clone();
            let result = cx
                .background_executor()
                .spawn(async move { std::fs::write(write_path, text) })
                .await;

            this.update_in(cx, |this, window, cx| match result {
                Ok(()) => {
                    this.finish_save(id, path.clone(), cx);
                    window
                        .push_notification(Notification::success(format!("Saved {filename}")), cx);
                }
                Err(error) => {
                    window.push_notification(
                        Notification::error(format!("Could not save {filename}: {error}")),
                        cx,
                    );
                }
            })
            .ok();
        })
        .detach();
    }

    fn finish_save(&mut self, id: EntityId, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(tab) = self
            .tabs
            .iter_mut()
            .find(|tab| tab.editor.entity_id() == id)
        {
            let language = language_for(&path);
            tab.title = file_name(&path).into();
            tab.editor.update(cx, |state, cx| {
                state.set_highlighter(editor_language_id(&language), cx);
            });
            tab.language = language;
            tab.path = Some(path);
            tab.dirty = false;
        }
        cx.notify();
    }

    fn on_close_tab(&mut self, _: &CloseTab, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ix) = self.active_tab {
            self.close_tab(ix, window, cx);
        }
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
        cx.notify();
    }

    fn activate_tab(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.active_tab = Some(ix);
        if let Some(tab) = self.tabs.get(ix) {
            let editor = tab.editor.clone();
            editor.update(cx, |state, cx| state.focus(window, cx));
            let position = editor.read(cx).cursor_position();
            self.cursor_line = position.line as usize + 1;
            self.cursor_col = position.character as usize + 1;
        }
        cx.notify();
    }

    fn close_tab(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some((id, dirty, title)) = self
            .tabs
            .get(ix)
            .map(|tab| (tab.editor.entity_id(), tab.dirty, tab.title.clone()))
        else {
            return;
        };

        if !dirty {
            self.remove_tab(id, window, cx);
            return;
        }

        let entity = cx.entity();
        window.open_alert_dialog(cx, move |alert, _window, _cx| {
            let entity = entity.clone();
            alert
                .confirm()
                .title("Unsaved Changes")
                .description(format!(
                    "\"{title}\" has unsaved changes. Close it without saving?"
                ))
                .ok_text("Close Without Saving")
                .ok_variant(ButtonVariant::Danger)
                .cancel_text("Keep Editing")
                .on_ok(move |_, window, cx| {
                    entity.update(cx, |this, cx| this.remove_tab(id, window, cx));
                    true
                })
        });
    }

    fn remove_tab(&mut self, id: EntityId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ix) = self
            .tabs
            .iter()
            .position(|tab| tab.editor.entity_id() == id)
        else {
            return;
        };

        self.tabs.remove(ix);
        self.active_tab = if self.tabs.is_empty() {
            None
        } else {
            let active = self.active_tab.unwrap_or(0);
            let active = if ix < active { active - 1 } else { active };
            Some(active.min(self.tabs.len() - 1))
        };

        if let Some(tab) = self.active_tab.and_then(|ix| self.tabs.get(ix)) {
            tab.editor.update(cx, |state, cx| state.focus(window, cx));
        }
        cx.notify();
    }

    fn set_dirty(&mut self, id: EntityId, dirty: bool, cx: &mut Context<Self>) {
        if let Some(tab) = self
            .tabs
            .iter_mut()
            .find(|tab| tab.editor.entity_id() == id)
            && tab.dirty != dirty
        {
            tab.dirty = dirty;
            cx.notify();
        }
    }

    fn sync_cursor(&mut self, id: EntityId, cx: &mut Context<Self>) {
        let Some(ix) = self.active_tab else {
            return;
        };
        let position = {
            let Some(tab) = self.tabs.get(ix) else {
                return;
            };
            if tab.editor.entity_id() != id {
                return;
            }
            tab.editor.read(cx).cursor_position()
        };

        self.cursor_line = position.line as usize + 1;
        self.cursor_col = position.character as usize + 1;
        cx.notify();
    }

    fn render_menu_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_tabs = !self.tabs.is_empty();
        let sidebar_visible = self.sidebar_visible;

        div()
            .h_flex()
            .flex_none()
            .h_9()
            .px_2()
            .gap_1()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("menu-file")
                    .small()
                    .compact()
                    .ghost()
                    .label("File")
                    .dropdown_menu(move |menu, _, _| {
                        menu.menu("New File", Box::new(NewFile))
                            .menu("Open…", Box::new(OpenFile))
                            .menu("Open Folder…", Box::new(OpenFolder))
                            .separator()
                            .menu("Save", Box::new(SaveFile))
                            .separator()
                            .menu_with_disabled("Close Tab", Box::new(CloseTab), !has_tabs)
                    }),
            )
            .child(
                Button::new("menu-edit")
                    .small()
                    .compact()
                    .ghost()
                    .label("Edit")
                    .dropdown_menu(|menu, _, _| {
                        menu.menu_with_disabled("Undo", Box::new(EditUndo), true)
                            .menu_with_disabled("Redo", Box::new(EditRedo), true)
                            .separator()
                            .menu_with_disabled("Cut", Box::new(EditCut), true)
                            .menu_with_disabled("Copy", Box::new(EditCopy), true)
                            .menu_with_disabled("Paste", Box::new(EditPaste), true)
                            .separator()
                            .menu_with_disabled("Select All", Box::new(EditSelectAll), true)
                    }),
            )
            .child(
                Button::new("menu-view")
                    .small()
                    .compact()
                    .ghost()
                    .label("View")
                    .dropdown_menu(move |menu, _, _| {
                        menu.menu_with_check("Sidebar", sidebar_visible, Box::new(ToggleSidebar))
                    }),
            )
            .child(div().flex_1())
            .child(
                Button::new("toggle-sidebar")
                    .small()
                    .compact()
                    .ghost()
                    .icon(IconName::PanelLeft)
                    .tooltip("Toggle Sidebar")
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_sidebar(cx))),
            )
    }

    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let active = self.active_tab;

        let tabs: Vec<Tab> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(ix, tab)| {
                let close_entity = entity.clone();
                Tab::new()
                    .label(tab.title.clone())
                    .suffix(
                        Button::new(("close-tab", ix))
                            .xsmall()
                            .ghost()
                            .icon(IconName::Close)
                            .on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                close_entity.update(cx, |this, cx| this.close_tab(ix, window, cx));
                            }),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.activate_tab(ix, window, cx);
                    }))
            })
            .collect();

        div()
            .flex_none()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                TabBar::new("workspace-tabs")
                    .selected_index(active.unwrap_or(0))
                    .children(tabs),
            )
    }

    fn render_workspace(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.sidebar_visible
            && let Some(root) = self.workspace_root.clone()
        {
            div().flex_1().min_h_0().child(
                h_resizable("workspace-panels")
                    .child(
                        resizable_panel()
                            .size(SIDEBAR_WIDTH)
                            .size_range(px(160.)..px(480.))
                            .child(self.render_sidebar(&root, cx)),
                    )
                    .child(resizable_panel().child(self.render_editor(cx))),
            )
        } else {
            div().flex_1().min_h_0().child(self.render_editor(cx))
        }
    }

    fn render_sidebar(&self, root: &Path, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self.tree_rows(root, 0, cx);

        div()
            .v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                div()
                    .h_flex()
                    .flex_none()
                    .h_9()
                    .px_3()
                    .items_center()
                    .gap_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(Icon::new(IconName::FolderOpen).with_size(Size::Small))
                    .child(div().text_sm().font_semibold().child(file_name(root))),
            )
            .child(
                div()
                    .id("explorer-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .py_1()
                    .children(rows),
            )
    }

    fn tree_rows(&self, dir: &Path, depth: usize, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut rows = Vec::new();

        for entry in read_dir(dir) {
            let is_dir = entry.is_dir();
            rows.push(self.tree_row(&entry, is_dir, depth, cx));
            if is_dir && self.expanded.contains(&entry) {
                rows.extend(self.tree_rows(&entry, depth + 1, cx));
            }
        }

        rows
    }

    fn tree_row(
        &self,
        path: &Path,
        is_dir: bool,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.selected_path.as_deref() == Some(path);
        let expanded = self.expanded.contains(path);
        let muted = cx.theme().muted_foreground;

        let leading = if is_dir {
            Icon::new(if expanded {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            })
            .with_size(Size::Small)
            .text_color(muted)
            .into_any_element()
        } else {
            div().w(px(14.)).into_any_element()
        };

        let type_icon = if is_dir {
            Icon::new(if expanded {
                IconName::FolderOpen
            } else {
                IconName::Folder
            })
        } else {
            Icon::new(IconName::FileText)
        }
        .with_size(Size::Small)
        .text_color(muted);

        let click_path = path.to_path_buf();
        let label = file_name(path);

        ListItem::new(path.display().to_string())
            .pl(px(8. + depth as f32 * 12.))
            .pr_2()
            .py_0p5()
            .gap_1()
            .selected(selected)
            .accessibility_label(label.clone())
            .child(leading)
            .child(type_icon)
            .child(div().text_sm().child(label))
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.selected_path = Some(click_path.clone());
                if is_dir {
                    if this.expanded.contains(&click_path) {
                        this.expanded.remove(&click_path);
                    } else {
                        this.expanded.insert(click_path.clone());
                    }
                } else if event.click_count() >= 2 {
                    this.request_open(click_path.clone(), window, cx);
                }
                cx.notify();
            }))
            .into_any_element()
    }

    fn render_editor(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.active() {
            Some(tab) => div()
                .size_full()
                .bg(cx.theme().background)
                .child(
                    Editor::new(&tab.editor)
                        .appearance(false)
                        .bordered(false)
                        .h_full(),
                )
                .into_any_element(),
            None => div()
                .flex()
                .size_full()
                .items_center()
                .justify_center()
                .text_color(cx.theme().muted_foreground)
                .child("No file open")
                .into_any_element(),
        }
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let language = self.active_language();
        let dirty = self.active_dirty();
        let divider = || div().h_3().w(px(1.)).flex_none().bg(cx.theme().border);

        let language_button = Button::new("language-mode")
            .xsmall()
            .ghost()
            .label(language)
            .dropdown_menu(move |menu, _, _| {
                LANGUAGES.iter().fold(menu, |menu, name| {
                    let entity = entity.clone();
                    let value: SharedString = (*name).into();
                    let item = PopupMenuItem::new(value.clone()).on_click(move |_, _, cx| {
                        let value = value.clone();
                        entity.update(cx, |this, cx| {
                            if let Some(tab) = this.active_tab.and_then(|ix| this.tabs.get_mut(ix))
                            {
                                tab.language = value.clone();
                                tab.editor.update(cx, |state, cx| {
                                    state.set_highlighter(editor_language_id(&value), cx);
                                });
                                cx.notify();
                            }
                        });
                    });
                    menu.item(item)
                })
            });

        let save_status = if dirty {
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .child(
                    Badge::new()
                        .dot()
                        .child(Icon::new(IconName::FileText).with_size(Size::XSmall)),
                )
                .child("Unsaved")
                .into_any_element()
        } else {
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .child(Icon::new(IconName::CircleCheck).with_size(Size::XSmall))
                .child("Saved")
                .into_any_element()
        };

        StatusBar::new()
            .left(div().child(format!("Ln {}, Col {}", self.cursor_line, self.cursor_col)))
            .left(divider())
            .left(div().child("UTF-8"))
            .right(language_button)
            .right(divider())
            .right(save_status)
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
            .child(self.render_menu_bar(cx))
            .child(self.render_tab_bar(cx))
            .child(self.render_workspace(cx))
            .child(self.render_status_bar(cx))
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn language_for(path: &Path) -> SharedString {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "Rust".into(),
        Some("toml") => "TOML".into(),
        Some("json") => "JSON".into(),
        Some("md") => "Markdown".into(),
        Some("js") => "JavaScript".into(),
        Some("ts") => "TypeScript".into(),
        Some("py") => "Python".into(),
        Some("html") | Some("htm") => "HTML".into(),
        Some("css") | Some("scss") => "CSS".into(),
        Some("php") | Some("phtml") => "PHP".into(),
        _ => "Plain Text".into(),
    }
}

fn editor_language_id(language: &str) -> &'static str {
    match language {
        "Rust" => "rust",
        "TOML" => "toml",
        "JSON" => "json",
        "Markdown" => "markdown",
        "JavaScript" => "javascript",
        "TypeScript" => "typescript",
        "Python" => "python",
        "HTML" => "html",
        "CSS" => "css",
        "PHP" => "php",
        _ => "plaintext",
    }
}

fn read_dir(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            dirs.push(path);
        } else {
            files.push(path);
        }
    }

    dirs.sort();
    files.sort();
    dirs.extend(files);
    dirs
}

fn path_from_file_url(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("file://")?;
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    path.starts_with('/')
        .then(|| PathBuf::from(percent_decode(path)))
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_digit(bytes[index + 1]), hex_digit(bytes[index + 2]))
        {
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn main() {
    let (open_tx, open_rx) = async_channel::unbounded::<Vec<PathBuf>>();

    let application = gpui_kit::application().with_assets(gpui_kit::assets::Assets);
    application.on_open_urls(move |urls| {
        let paths: Vec<PathBuf> = urls
            .iter()
            .filter_map(|url| path_from_file_url(url))
            .collect();
        if !paths.is_empty() {
            let _ = open_tx.try_send(paths);
        }
    });

    application.run(move |cx| {
        gpui_kit::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);

        let initial_folder = std::env::args()
            .nth(1)
            .map(PathBuf::from)
            .filter(|path| path.is_dir());

        cx.bind_keys([
            KeyBinding::new("cmd-n", NewFile, None),
            KeyBinding::new("cmd-o", OpenFile, None),
            KeyBinding::new("cmd-s", SaveFile, None),
            KeyBinding::new("cmd-w", CloseTab, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("cmd-b", ToggleSidebar, None),
        ]);

        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Tinytext".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, move |window, cx| {
            let app = cx.new(|cx| TinytextApp::new(initial_folder.clone(), cx));
            let focus_handle = app.read(cx).focus_handle.clone();
            focus_handle.focus(window, cx);

            app.update(cx, |_this, cx| {
                cx.spawn_in(window, async move |this, cx| {
                    while let Ok(paths) = open_rx.recv().await {
                        this.update_in(cx, |this, window, cx| {
                            for path in paths {
                                this.request_open(path, window, cx);
                            }
                        })
                        .ok();
                    }
                })
                .detach();
            });

            app
        })
        .expect("failed to open window");
    });
}
