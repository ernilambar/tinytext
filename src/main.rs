use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, Size,
    badge::Badge,
    button::{Button, ButtonVariants as _},
    list::ListItem,
    menu::{DropdownMenu as _, PopupMenuItem},
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
        SaveFile,
        CloseTab,
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
const LANGUAGES: [&str; 8] = [
    "Plain Text",
    "Rust",
    "TOML",
    "JSON",
    "Markdown",
    "JavaScript",
    "TypeScript",
    "Python",
];

struct OpenTab {
    path: Option<PathBuf>,
    title: SharedString,
    content: String,
    language: SharedString,
    dirty: bool,
}

struct TinytextApp {
    focus_handle: FocusHandle,
    workspace_root: PathBuf,
    expanded: HashSet<PathBuf>,
    selected_path: Option<PathBuf>,
    tabs: Vec<OpenTab>,
    active_tab: Option<usize>,
    sidebar_visible: bool,
    cursor_line: usize,
    cursor_col: usize,
}

impl TinytextApp {
    fn new(cx: &mut Context<Self>) -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut expanded = HashSet::new();
        expanded.insert(workspace_root.clone());

        Self {
            focus_handle: cx.focus_handle(),
            workspace_root,
            expanded,
            selected_path: None,
            tabs: Vec::new(),
            active_tab: None,
            sidebar_visible: true,
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

    fn on_new_file(&mut self, _: &NewFile, _window: &mut Window, cx: &mut Context<Self>) {
        self.tabs.push(OpenTab {
            path: None,
            title: "Untitled".into(),
            content: String::new(),
            language: "Plain Text".into(),
            dirty: true,
        });
        self.active_tab = Some(self.tabs.len() - 1);
        self.cursor_line = 1;
        self.cursor_col = 1;
        cx.notify();
    }

    fn on_open_file(&mut self, _: &OpenFile, _window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Open File".into()),
        });

        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await {
                this.update(cx, |this, cx| {
                    for path in paths {
                        this.open_path(path, cx);
                    }
                })
                .ok();
            }
        })
        .detach();
    }

    fn on_save_file(&mut self, _: &SaveFile, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(ix) = self.active_tab else {
            return;
        };

        match self.tabs[ix].path.clone() {
            Some(path) => self.save_tab_to(ix, path, cx),
            None => {
                let directory = self.workspace_root.clone();
                let receiver = cx.prompt_for_new_path(&directory, Some("untitled.txt"));

                cx.spawn(async move |this, cx| {
                    if let Ok(Ok(Some(path))) = receiver.await {
                        this.update(cx, |this, cx| this.save_tab_to(ix, path, cx))
                            .ok();
                    }
                })
                .detach();
            }
        }
    }

    fn on_close_tab(&mut self, _: &CloseTab, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ix) = self.active_tab {
            self.close_tab(ix, cx);
        }
    }

    fn on_toggle_sidebar(
        &mut self,
        _: &ToggleSidebar,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar_visible = !self.sidebar_visible;
        cx.notify();
    }

    fn open_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(ix) = self
            .tabs
            .iter()
            .position(|tab| tab.path.as_deref() == Some(path.as_path()))
        {
            self.active_tab = Some(ix);
            cx.notify();
            return;
        }

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        self.tabs.push(OpenTab {
            language: language_for(&path),
            title: file_name(&path).into(),
            path: Some(path),
            content,
            dirty: false,
        });
        self.active_tab = Some(self.tabs.len() - 1);
        self.cursor_line = 1;
        self.cursor_col = 1;
        cx.notify();
    }

    fn save_tab_to(&mut self, ix: usize, path: PathBuf, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(ix) else {
            return;
        };

        if std::fs::write(&path, tab.content.as_bytes()).is_ok() {
            tab.title = file_name(&path).into();
            tab.language = language_for(&path);
            tab.path = Some(path);
            tab.dirty = false;
        }
        cx.notify();
    }

    fn close_tab(&mut self, ix: usize, cx: &mut Context<Self>) {
        if ix >= self.tabs.len() {
            return;
        }

        self.tabs.remove(ix);
        self.active_tab = if self.tabs.is_empty() {
            None
        } else {
            let active = self.active_tab.unwrap_or(0);
            let active = if ix < active { active - 1 } else { active };
            Some(active.min(self.tabs.len() - 1))
        };
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
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_visible = !this.sidebar_visible;
                        cx.notify();
                    })),
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
                            .on_click(move |_, _, cx| {
                                cx.stop_propagation();
                                close_entity.update(cx, |this, cx| this.close_tab(ix, cx));
                            }),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active_tab = Some(ix);
                        cx.notify();
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
        if self.sidebar_visible {
            div().flex_1().min_h_0().child(
                h_resizable("workspace-panels")
                    .child(
                        resizable_panel()
                            .size(SIDEBAR_WIDTH)
                            .size_range(px(160.)..px(480.))
                            .child(self.render_sidebar(cx)),
                    )
                    .child(resizable_panel().child(self.render_editor(cx))),
            )
        } else {
            div().flex_1().min_h_0().child(self.render_editor(cx))
        }
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let root = self.workspace_root.clone();
        let rows = self.tree_rows(&root, 0, cx);

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
                    .child(div().text_sm().font_semibold().child(file_name(&root))),
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
            .on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
                this.selected_path = Some(click_path.clone());
                if is_dir {
                    if this.expanded.contains(&click_path) {
                        this.expanded.remove(&click_path);
                    } else {
                        this.expanded.insert(click_path.clone());
                    }
                } else if event.click_count() >= 2 {
                    this.open_path(click_path.clone(), cx);
                }
                cx.notify();
            }))
            .into_any_element()
    }

    fn render_editor(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.active() {
            Some(tab) => div()
                .id("editor-scroll")
                .size_full()
                .overflow_y_scroll()
                .p_3()
                .font_family("Menlo")
                .text_sm()
                .children(self.editor_lines(&tab.content, cx))
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

    fn editor_lines(&self, content: &str, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let gutter = cx.theme().muted_foreground;

        content
            .lines()
            .enumerate()
            .map(|(ix, line)| {
                div()
                    .h_flex()
                    .h(px(18.))
                    .gap_3()
                    .child(
                        div()
                            .w(px(32.))
                            .flex_none()
                            .text_color(gutter)
                            .child(format!("{}", ix + 1)),
                    )
                    .child(div().child(line.to_string()))
                    .into_any_element()
            })
            .collect()
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
                                tab.language = value;
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
            .on_action(cx.listener(Self::on_save_file))
            .on_action(cx.listener(Self::on_close_tab))
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
        _ => "Plain Text".into(),
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

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);

            let options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Tinytext".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                let app = cx.new(TinytextApp::new);
                let focus_handle = app.read(cx).focus_handle.clone();
                focus_handle.focus(window, cx);
                app
            })
            .expect("failed to open window");
        });
}
