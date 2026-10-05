use std::path::Path;

use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, Size,
    badge::Badge,
    button::{Button, ButtonVariants as _},
    input::Editor,
    list::ListItem,
    menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem},
    resizable::{h_resizable, resizable_panel},
    status_bar::StatusBar,
    tab::{Tab, TabBar},
};
use gpui_kit::*;

use crate::language::{LANGUAGES, editor_language_id};
use crate::paths::{file_name, read_dir};
use crate::{
    About, CloseTab, EditCopy, EditCut, EditPaste, EditRedo, EditSelectAll, EditUndo, InstallCli,
    NewFile, OpenFile, OpenFolder, Quit, SaveFile, ToggleSidebar,
};

use super::TinytextApp;

const SIDEBAR_WIDTH: Pixels = px(240.);

impl TinytextApp {
    pub(super) fn render_menu_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .separator()
                            .menu("Quit", Box::new(Quit))
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
            .child(
                Button::new("menu-help")
                    .small()
                    .compact()
                    .ghost()
                    .label("Help")
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("Install \"tinytext\" Command in PATH", Box::new(InstallCli))
                            .separator()
                            .menu("About Tinytext", Box::new(About))
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

    pub(super) fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, _, _, _cx| {
                            this.context_tab = Some(ix);
                        }),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.activate_tab(ix, window, cx);
                    }))
            })
            .collect();

        let menu_entity = entity.clone();
        div()
            .id("workspace-tab-bar")
            .flex_none()
            .border_b_1()
            .border_color(cx.theme().border)
            .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, _, _cx| {
                if event.button == MouseButton::Right {
                    this.context_tab = None;
                }
            }))
            .context_menu(move |menu, _, cx| {
                let Some(ix) = menu_entity.read(cx).context_tab else {
                    return menu;
                };
                let (tab_count, has_deleted) = {
                    let app = menu_entity.read(cx);
                    (
                        app.tabs.len(),
                        app.tabs
                            .iter()
                            .any(|tab| tab.path.as_deref().is_some_and(|path| !path.exists())),
                    )
                };
                if ix >= tab_count {
                    return menu;
                }

                let close_entity = menu_entity.clone();
                let others_entity = menu_entity.clone();
                let right_entity = menu_entity.clone();
                let deleted_entity = menu_entity.clone();

                menu.item(
                    PopupMenuItem::new("Close Tab").on_click(move |_, window, cx| {
                        close_entity.update(cx, |this, cx| this.close_tab(ix, window, cx));
                    }),
                )
                .separator()
                .item(
                    PopupMenuItem::new("Close Other Tabs")
                        .disabled(tab_count <= 1)
                        .on_click(move |_, window, cx| {
                            others_entity
                                .update(cx, |this, cx| this.close_other_tabs(ix, window, cx));
                        }),
                )
                .item(
                    PopupMenuItem::new("Close Tabs to the Right")
                        .disabled(ix + 1 >= tab_count)
                        .on_click(move |_, window, cx| {
                            right_entity
                                .update(cx, |this, cx| this.close_tabs_to_right(ix, window, cx));
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new("Close Tabs with Deleted Files")
                        .disabled(!has_deleted)
                        .on_click(move |_, window, cx| {
                            deleted_entity.update(cx, |this, cx| {
                                this.close_tabs_with_deleted_files(window, cx)
                            });
                        }),
                )
            })
            .child(
                TabBar::new("workspace-tabs")
                    .selected_index(active.unwrap_or(0))
                    .children(tabs),
            )
    }

    pub(super) fn render_workspace(&self, cx: &mut Context<Self>) -> impl IntoElement {
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

    pub(super) fn render_sidebar(&self, root: &Path, cx: &mut Context<Self>) -> impl IntoElement {
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

    pub(super) fn tree_rows(
        &self,
        dir: &Path,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
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

    pub(super) fn tree_row(
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
            .selected(selected)
            .accessibility_label(label.clone())
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_1()
                    .child(leading)
                    .child(type_icon)
                    .child(div().text_sm().child(label)),
            )
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
                this.save_session();
                cx.notify();
            }))
            .into_any_element()
    }

    pub(super) fn render_editor(&self, cx: &mut Context<Self>) -> AnyElement {
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

    pub(super) fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
