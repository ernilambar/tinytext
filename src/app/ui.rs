use std::path::Path;

use gpui_kit::base::{InteractiveElementExt as _, StyledExt as _};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, Size,
    badge::Badge,
    button::{Button, ButtonVariants as _},
    input::{self, Editor},
    list::ListItem,
    menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem},
    resizable::{h_resizable, resizable_panel},
    scroll::ScrollableElement as _,
    status_bar::StatusBar,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::file_icons::{file_icon, folder_icon};
use crate::language::{LANGUAGES, editor_language_id};
use crate::paths::{file_name, read_dir};
use crate::{CopyFilePath, CopyRelativePath, RevealInFinder};

use super::TinytextApp;

const SIDEBAR_WIDTH: Pixels = px(240.);
/// Bounds on a tab's width. Tabs share the strip equally and shrink together as
/// more open, giving way no further than `TAB_MIN_WIDTH`; past that the strip
/// scrolls horizontally instead of squeezing labels into nothing.
const TAB_MIN_WIDTH: Pixels = px(100.);
const TAB_MAX_WIDTH: Pixels = px(200.);

const TAB_HEIGHT: Pixels = px(32.);
const TAB_FONT_SIZE: Pixels = px(12.);
/// Thickness of the accent line along the active tab's top edge.
const TAB_ACCENT_HEIGHT: Pixels = px(2.);

/// Group name shared by every tab, so its close button can react to hover.
const TAB_GROUP: &str = "tab";

/// A tab's status dot, drawn in the close button's slot and hidden on hover so
/// the button shows through.
fn status_dot(visible: bool, color: Hsla) -> Div {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .group_hover(TAB_GROUP, |style| style.invisible())
        .when(visible, |this| {
            this.child(div().size(px(6.)).rounded_full().bg(color))
        })
}

impl TinytextApp {
    pub(super) fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let active = self.active_tab;
        let theme = cx.theme();
        let dirty_color = theme.foreground;
        let deleted_color = theme.red;
        let border_color = theme.border;
        let bar_bg = theme.tab_bar;
        let active_bg = theme.tab_active;
        // The theme's `border` and `secondary_hover` match the bar's own colour,
        // so separators take the editor background and hover lifts the bar.
        let separator_color = theme.background;
        let hover_bg = bar_bg.blend(theme.foreground.opacity(0.06));
        let fg = theme.tab_foreground;
        let active_fg = theme.tab_active_foreground;
        let accent = theme.blue;

        let tabs: Vec<Stateful<Div>> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(ix, tab)| {
                let close_entity = entity.clone();
                let selected = active == Some(ix);
                let deleted = tab.path.as_deref().is_some_and(|path| !path.exists());
                let (dot, dot_color) = if deleted {
                    (true, deleted_color)
                } else {
                    (tab.dirty, dirty_color)
                };
                div()
                    .id(("tab", ix))
                    .group(TAB_GROUP)
                    .relative()
                    .h_flex()
                    // Grow into an equal share of the strip (basis 0 ignores the
                    // title's natural size), so tabs shrink in step as more open.
                    // `min_w` floors that share; past it the strip scrolls.
                    .flex_1()
                    .min_w(TAB_MIN_WIDTH)
                    .max_w(TAB_MAX_WIDTH)
                    .h(TAB_HEIGHT)
                    .pl_3()
                    .pr_1()
                    .gap_1()
                    .text_size(TAB_FONT_SIZE)
                    .border_r_1()
                    .border_color(separator_color)
                    .map(|this| {
                        if selected {
                            this.bg(active_bg).text_color(active_fg).child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .right_0()
                                    .h(TAB_ACCENT_HEIGHT)
                                    .bg(accent),
                            )
                        } else {
                            this.bg(bar_bg)
                                .border_b_1()
                                .text_color(fg)
                                .hover(|style| style.bg(hover_bg).text_color(active_fg))
                        }
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(tab.title.clone()),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_none()
                            .child(
                                div()
                                    .invisible()
                                    .group_hover(TAB_GROUP, |style| style.visible())
                                    .child(
                                        Button::new(("close-tab", ix))
                                            .xsmall()
                                            .ghost()
                                            .icon(IconName::Close)
                                            .on_click(move |_, window, cx| {
                                                cx.stop_propagation();
                                                close_entity.update(cx, |this, cx| {
                                                    this.close_tab(ix, window, cx)
                                                });
                                            }),
                                    ),
                            )
                            .child(status_dot(dot, dot_color)),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.activate_tab(ix, window, cx);
                    }))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, _, _, _cx| {
                            this.context_tab = Some(ix);
                        }),
                    )
            })
            .collect();

        let titles: Vec<SharedString> = self.tabs.iter().map(|tab| tab.title.clone()).collect();
        let overflow_entity = entity.clone();
        let menu_entity = entity.clone();
        div()
            .id("workspace-tab-bar")
            .flex_none()
            .relative()
            .h_flex()
            .bg(bar_bg)
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
            // The bar's bottom edge, painted beneath the tabs. Inactive tabs draw
            // their own; the active tab covers it to blend into the editor.
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(px(1.))
                    .bg(border_color),
            )
            .child(
                div()
                    .id("workspace-tabs")
                    .flex_1()
                    .min_w_0()
                    .h_flex()
                    .overflow_x_scroll()
                    .lock_scroll_axis()
                    .track_scroll(&self.tab_scroll)
                    .children(tabs),
            )
            .child(
                div().flex_none().px_1().child(
                    Button::new("tab-overflow")
                        .xsmall()
                        .ghost()
                        .dropdown_caret(true)
                        .dropdown_menu(move |menu, _, _| {
                            titles.iter().enumerate().fold(
                                menu.scrollable(true),
                                |menu, (ix, title)| {
                                    let entity = overflow_entity.clone();
                                    menu.item(
                                        PopupMenuItem::new(title.clone())
                                            .checked(active == Some(ix))
                                            .on_click(move |_, window, cx| {
                                                entity.update(cx, |this, cx| {
                                                    this.activate_tab(ix, window, cx)
                                                });
                                            }),
                                    )
                                },
                            )
                        })
                        .anchor(Anchor::TopRight),
                ),
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
                    .child(resizable_panel().child(self.render_editor_panel(cx))),
            )
        } else {
            div().flex_1().min_h_0().child(self.render_editor_panel(cx))
        }
    }

    pub(super) fn render_editor_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .min_w_0()
            .child(self.render_tab_bar(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_editor_for(self.active_tab, cx)),
            )
    }

    pub(super) fn render_sidebar(&self, root: &Path, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self.tree_rows(root, 0, cx);
        let header_entity = cx.entity();
        let root_path = root.to_path_buf();

        div()
            .v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                div()
                    .id("sidebar-header")
                    .h_flex()
                    .flex_none()
                    .h_9()
                    .px_3()
                    .items_center()
                    .gap_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(Icon::new(IconName::FolderOpen).with_size(Size::Small))
                    .child(div().text_sm().font_semibold().child(file_name(root)))
                    .context_menu(move |menu, _, cx| {
                        let can_paste = header_entity.read(cx).file_clipboard.is_some();
                        let target = root_path.clone();

                        let new_file_entity = header_entity.clone();
                        let new_file_dir = target.clone();
                        let new_folder_entity = header_entity.clone();
                        let new_folder_dir = target.clone();
                        let paste_entity = header_entity.clone();
                        let paste_dir = target.clone();
                        let path_entity = header_entity.clone();
                        let path_value = target.clone();
                        let relative_entity = header_entity.clone();
                        let relative_value = target.clone();
                        let reveal_entity = header_entity.clone();
                        let reveal_path = target;

                        menu.item(
                            PopupMenuItem::new("New File…").on_click(move |_, window, cx| {
                                new_file_entity.update(cx, |this, cx| {
                                    this.create_entry(new_file_dir.clone(), false, window, cx)
                                });
                            }),
                        )
                        .item(
                            PopupMenuItem::new("New Folder…").on_click(move |_, window, cx| {
                                new_folder_entity.update(cx, |this, cx| {
                                    this.create_entry(new_folder_dir.clone(), true, window, cx)
                                });
                            }),
                        )
                        .separator()
                        .item(PopupMenuItem::new("Paste").disabled(!can_paste).on_click(
                            move |_, window, cx| {
                                paste_entity.update(cx, |this, cx| {
                                    this.paste_clipboard(paste_dir.clone(), window, cx)
                                });
                            },
                        ))
                        .separator()
                        .item(
                            PopupMenuItem::new("Copy Path").on_click(move |_, _window, cx| {
                                path_entity.update(cx, |this, cx| {
                                    this.copy_path_to_clipboard(&path_value, cx)
                                });
                            }),
                        )
                        .item(PopupMenuItem::new("Copy Relative Path").on_click(
                            move |_, _window, cx| {
                                relative_entity.update(cx, |this, cx| {
                                    this.copy_relative_path(&relative_value, cx)
                                });
                            },
                        ))
                        .separator()
                        .item(
                            PopupMenuItem::new("Reveal in Finder").on_click(
                                move |_, _window, cx| {
                                    reveal_entity.update(cx, |this, cx| {
                                        this.reveal_in_finder(&reveal_path, cx)
                                    });
                                },
                            ),
                        )
                    }),
            )
            .child(
                div()
                    .id("explorer-scroll")
                    .flex_1()
                    .min_h_0()
                    .py_1()
                    .overflow_scrollbar()
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
            Icon::empty().data(folder_icon())
        } else {
            Icon::empty().data(file_icon(path))
        }
        .with_size(Size::Small);

        let click_path = path.to_path_buf();
        let label = file_name(path);
        let menu_entity = cx.entity();
        let menu_path = path.to_path_buf();

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
                    .child(div().text_sm().whitespace_nowrap().child(label)),
            )
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.selected_path = Some(click_path.clone());
                if is_dir {
                    if this.expanded.contains(&click_path) {
                        this.expanded.remove(&click_path);
                    } else {
                        this.expanded.insert(click_path.clone());
                    }
                } else if event.click_count() == 1 {
                    this.request_open(click_path.clone(), window, cx);
                }
                this.save_session();
                cx.notify();
            }))
            .context_menu(move |menu, _, cx| {
                let paste_dir = if is_dir {
                    menu_path.clone()
                } else {
                    menu_path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_default()
                };
                let can_paste = menu_entity.read(cx).file_clipboard.is_some();

                let menu = if is_dir {
                    let new_file_entity = menu_entity.clone();
                    let new_file_dir = menu_path.clone();
                    let new_folder_entity = menu_entity.clone();
                    let new_folder_dir = menu_path.clone();
                    menu.item(
                        PopupMenuItem::new("New File…").on_click(move |_, window, cx| {
                            new_file_entity.update(cx, |this, cx| {
                                this.create_entry(new_file_dir.clone(), false, window, cx)
                            });
                        }),
                    )
                    .item(
                        PopupMenuItem::new("New Folder…").on_click(move |_, window, cx| {
                            new_folder_entity.update(cx, |this, cx| {
                                this.create_entry(new_folder_dir.clone(), true, window, cx)
                            });
                        }),
                    )
                    .separator()
                } else {
                    let open_entity = menu_entity.clone();
                    let open_path = menu_path.clone();
                    menu.item(PopupMenuItem::new("Open").on_click(move |_, window, cx| {
                        open_entity.update(cx, |this, cx| {
                            this.request_open(open_path.clone(), window, cx)
                        });
                    }))
                    .separator()
                };

                let reveal_entity = menu_entity.clone();
                let reveal_path = menu_path.clone();
                let cut_entity = menu_entity.clone();
                let cut_path = menu_path.clone();
                let copy_entity = menu_entity.clone();
                let copy_path = menu_path.clone();
                let paste_entity = menu_entity.clone();
                let name_entity = menu_entity.clone();
                let name_value = menu_path.clone();
                let path_entity = menu_entity.clone();
                let path_value = menu_path.clone();
                let relative_entity = menu_entity.clone();
                let relative_value = menu_path.clone();
                let rename_entity = menu_entity.clone();
                let rename_path = menu_path.clone();
                let duplicate_entity = menu_entity.clone();
                let duplicate_path = menu_path.clone();
                let move_entity = menu_entity.clone();
                let move_path = menu_path.clone();
                let delete_entity = menu_entity.clone();
                let delete_path = menu_path.clone();

                menu.item(
                    PopupMenuItem::new("Reveal in Finder").on_click(move |_, _window, cx| {
                        reveal_entity
                            .update(cx, |this, cx| this.reveal_in_finder(&reveal_path, cx));
                    }),
                )
                .separator()
                .item(PopupMenuItem::new("Cut").on_click(move |_, _window, cx| {
                    cut_entity.update(cx, |this, cx| {
                        this.set_file_clipboard(cut_path.clone(), true, cx)
                    });
                }))
                .item(PopupMenuItem::new("Copy").on_click(move |_, _window, cx| {
                    copy_entity.update(cx, |this, cx| {
                        this.set_file_clipboard(copy_path.clone(), false, cx)
                    });
                }))
                .item(PopupMenuItem::new("Paste").disabled(!can_paste).on_click(
                    move |_, window, cx| {
                        paste_entity.update(cx, |this, cx| {
                            this.paste_clipboard(paste_dir.clone(), window, cx)
                        });
                    },
                ))
                .separator()
                .item(
                    PopupMenuItem::new("Copy Name").on_click(move |_, _window, cx| {
                        name_entity
                            .update(cx, |this, cx| this.copy_name_to_clipboard(&name_value, cx));
                    }),
                )
                .item(
                    PopupMenuItem::new("Copy Path").on_click(move |_, _window, cx| {
                        path_entity
                            .update(cx, |this, cx| this.copy_path_to_clipboard(&path_value, cx));
                    }),
                )
                .item(
                    PopupMenuItem::new("Copy Relative Path").on_click(move |_, _window, cx| {
                        relative_entity
                            .update(cx, |this, cx| this.copy_relative_path(&relative_value, cx));
                    }),
                )
                .separator()
                .item(
                    PopupMenuItem::new("Rename…").on_click(move |_, window, cx| {
                        rename_entity.update(cx, |this, cx| {
                            this.rename_entry(rename_path.clone(), window, cx)
                        });
                    }),
                )
                .item(
                    PopupMenuItem::new("Duplicate").on_click(move |_, window, cx| {
                        duplicate_entity.update(cx, |this, cx| {
                            this.duplicate_entry(duplicate_path.clone(), window, cx)
                        });
                    }),
                )
                .item(PopupMenuItem::new("Move…").on_click(move |_, window, cx| {
                    move_entity.update(cx, |this, cx| {
                        this.move_entry(move_path.clone(), window, cx)
                    });
                }))
                .item(PopupMenuItem::new("Delete").on_click(move |_, window, cx| {
                    delete_entity.update(cx, |this, cx| {
                        this.delete_entry(delete_path.clone(), window, cx)
                    });
                }))
            })
            .into_any_element()
    }

    pub(super) fn render_editor_for(
        &self,
        ix: Option<usize>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match ix.and_then(|ix| self.tabs.get(ix)) {
            Some(tab) => {
                let has_path = tab.path.is_some();
                div()
                    .size_full()
                    .bg(cx.theme().background)
                    .child(
                        Editor::new(&tab.editor)
                            .appearance(false)
                            .bordered(false)
                            .when_some(
                                self.settings.editor.font_family.clone(),
                                |editor, family| editor.font_family(family),
                            )
                            .when_some(self.settings.editor.font_size(), |editor, size| {
                                editor.text_size(px(size))
                            })
                            .when_some(self.settings.editor.font_weight(), |editor, weight| {
                                editor.font_weight(FontWeight::from(weight))
                            })
                            .context_menu(move |menu, _, _| {
                                // Built while the editor state is mid-update, so it must not
                                // read that state. Cut and Copy no-op without a selection.
                                menu.menu("Cut", Box::new(input::Cut))
                                    .menu("Copy", Box::new(input::Copy))
                                    .menu("Paste", Box::new(input::Paste))
                                    .separator()
                                    .menu("Select All", Box::new(input::SelectAll))
                                    .separator()
                                    .menu_with_disabled(
                                        "Reveal in Finder",
                                        !has_path,
                                        Box::new(RevealInFinder),
                                    )
                                    .menu_with_disabled(
                                        "Copy Path",
                                        !has_path,
                                        Box::new(CopyFilePath),
                                    )
                                    .menu_with_disabled(
                                        "Copy Relative Path",
                                        !has_path,
                                        Box::new(CopyRelativePath),
                                    )
                            })
                            .h_full(),
                    )
                    .into_any_element()
            }
            None => div()
                .flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(super::logo(128.).grayscale(true).opacity(0.5))
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
