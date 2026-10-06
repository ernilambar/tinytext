use std::path::{Path, PathBuf};

use gpui_kit::component::{
    WindowExt as _,
    button::ButtonVariant,
    input::{Input, InputState},
    notification::Notification,
};
use gpui_kit::*;

use crate::language::{editor_language_id, language_for};
use crate::paths::{copy_entry, file_name, is_valid_entry_name, remap_prefix};
use crate::{NewFile, OpenFile, OpenFolder, SaveAll, SaveFile, SaveFileAs};

use super::{FileClipboard, TinytextApp};

/// Copy for the single-field name prompt shared by create and rename.
struct NamePrompt {
    title: &'static str,
    description: String,
    initial: String,
    ok_text: &'static str,
}

impl TinytextApp {
    pub(crate) fn request_open(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if path.is_dir() {
            self.open_folder(path, cx);
            return;
        }

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

    pub(super) fn on_new_file(&mut self, _: &NewFile, window: &mut Window, cx: &mut Context<Self>) {
        self.add_tab(None, String::new(), window, cx);
    }

    pub(super) fn on_open_file(
        &mut self,
        _: &OpenFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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

    pub(super) fn on_open_folder(
        &mut self,
        _: &OpenFolder,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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

    pub(super) fn open_folder(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.workspace_root = Some(path.clone());
        self.expanded.clear();
        self.expanded.insert(path);
        self.selected_path = None;
        self.sidebar_visible = true;
        self.save_session();
        cx.notify();
    }

    pub(super) fn on_save_file(
        &mut self,
        _: &SaveFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ix) = self.focused_tab() else {
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

    pub(super) fn on_save_file_as(
        &mut self,
        _: &SaveFileAs,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ix) = self.focused_tab() else {
            return;
        };
        let Some(tab) = self.tabs.get(ix) else {
            return;
        };
        let id = tab.editor.entity_id();
        let suggested = tab
            .path
            .as_deref()
            .map(file_name)
            .unwrap_or_else(|| "untitled.txt".to_string());
        let directory = tab
            .path
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .or_else(|| self.workspace_root.clone())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let receiver = cx.prompt_for_new_path(&directory, Some(suggested.as_str()));

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

    /// Saves every edited tab that still points at a file on disk. Untitled and
    /// deleted files are left alone, the latter so a save never recreates a file
    /// the user removed.
    pub(super) fn on_save_all(&mut self, _: &SaveAll, window: &mut Window, cx: &mut Context<Self>) {
        let targets: Vec<(EntityId, PathBuf)> = self
            .tabs
            .iter()
            .filter(|tab| tab.dirty)
            .filter_map(|tab| tab.path.clone().map(|path| (tab.editor.entity_id(), path)))
            .filter(|(_, path)| path.exists())
            .collect();

        for (id, path) in targets {
            self.save_editor(id, path, window, cx);
        }
    }

    pub(super) fn save_editor(
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
                    this.reload_settings_if(&path, window, cx);
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

    pub(super) fn finish_save(&mut self, id: EntityId, path: PathBuf, cx: &mut Context<Self>) {
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
        self.save_session();
        cx.notify();
    }

    /// Prompts for a single name with a text field and runs `on_confirm` with
    /// the trimmed value when the field is not empty.
    fn prompt_for_name<F>(
        &mut self,
        prompt: NamePrompt,
        window: &mut Window,
        cx: &mut Context<Self>,
        on_confirm: F,
    ) where
        F: Fn(&mut Self, String, &mut Window, &mut Context<Self>) + 'static,
    {
        let NamePrompt {
            title,
            description,
            initial,
            ok_text,
        } = prompt;
        let input = cx.new(|cx| {
            let state = InputState::new(window, cx).placeholder("Name");
            if initial.is_empty() {
                state
            } else {
                state.default_value(initial.clone())
            }
        });
        let value_input = input.clone();
        let focus_input = input.clone();
        let entity = cx.entity();
        let on_confirm = std::rc::Rc::new(on_confirm);

        window.open_alert_dialog(cx, move |alert, _window, _cx| {
            let value_input = value_input.clone();
            let entity = entity.clone();
            let on_confirm = on_confirm.clone();
            alert
                .confirm()
                .title(title)
                .description(description.clone())
                .child(div().w_full().child(Input::new(&input).w_full()))
                .ok_text(ok_text)
                .cancel_text("Cancel")
                .on_ok(move |_, window, cx| {
                    let name = value_input.read(cx).value().trim().to_string();
                    if name.is_empty() {
                        return false;
                    }
                    entity.update(cx, |this, cx| on_confirm(this, name, window, cx));
                    true
                })
        });

        window.defer(cx, move |window, cx| {
            focus_input.update(cx, |state, cx| state.focus(window, cx));
        });
    }

    pub(super) fn create_entry(
        &mut self,
        parent: PathBuf,
        is_dir: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (title, ok_text) = if is_dir {
            ("New Folder", "Create Folder")
        } else {
            ("New File", "Create File")
        };
        let description = format!("Create in {}", file_name(&parent));

        self.prompt_for_name(
            NamePrompt {
                title,
                description,
                initial: String::new(),
                ok_text,
            },
            window,
            cx,
            move |this, name, window, cx| {
                if !is_valid_entry_name(&name) {
                    window.push_notification(
                        Notification::error(format!("Invalid name \"{name}\"")),
                        cx,
                    );
                    return;
                }
                let path = parent.join(&name);
                if path.exists() {
                    window.push_notification(
                        Notification::error(format!("\"{name}\" already exists")),
                        cx,
                    );
                    return;
                }

                let result = if is_dir {
                    std::fs::create_dir(&path)
                } else {
                    std::fs::write(&path, "")
                };
                if let Err(error) = result {
                    window.push_notification(
                        Notification::error(format!("Could not create {name}: {error}")),
                        cx,
                    );
                    return;
                }

                this.expanded.insert(parent.clone());
                if is_dir {
                    this.expanded.insert(path.clone());
                }
                this.selected_path = Some(path.clone());
                this.save_session();
                if is_dir {
                    cx.notify();
                } else {
                    this.request_open(path, window, cx);
                }
            },
        );
    }

    pub(super) fn rename_entry(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = file_name(&path);
        let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let description = format!("Rename \"{name}\"");

        self.prompt_for_name(
            NamePrompt {
                title: "Rename",
                description,
                initial: name,
                ok_text: "Rename",
            },
            window,
            cx,
            move |this, new_name, window, cx| {
                if !is_valid_entry_name(&new_name) {
                    window.push_notification(
                        Notification::error(format!("Invalid name \"{new_name}\"")),
                        cx,
                    );
                    return;
                }
                let new_path = parent.join(&new_name);
                if new_path == path {
                    return;
                }
                if new_path.exists() {
                    window.push_notification(
                        Notification::error(format!("\"{new_name}\" already exists")),
                        cx,
                    );
                    return;
                }

                match std::fs::rename(&path, &new_path) {
                    Ok(()) => {
                        this.after_rename(&path, &new_path, cx);
                        window.push_notification(
                            Notification::success(format!("Renamed to {new_name}")),
                            cx,
                        );
                    }
                    Err(error) => window.push_notification(
                        Notification::error(format!("Could not rename: {error}")),
                        cx,
                    ),
                }
            },
        );
    }

    /// Moves every open tab, expansion entry, and the workspace root from the
    /// old path prefix to the new one after a rename or move.
    fn after_rename(&mut self, old: &Path, new: &Path, cx: &mut Context<Self>) {
        for tab in &mut self.tabs {
            let Some(current) = tab.path.clone() else {
                continue;
            };
            let Some(mapped) = remap_prefix(&current, old, new) else {
                continue;
            };
            let language = language_for(&mapped);
            tab.title = file_name(&mapped).into();
            tab.editor.update(cx, |state, cx| {
                state.set_highlighter(editor_language_id(&language), cx);
            });
            tab.language = language;
            tab.path = Some(mapped);
        }

        let remap = |path: &PathBuf| remap_prefix(path, old, new).unwrap_or_else(|| path.clone());
        self.expanded = self.expanded.iter().map(remap).collect();
        self.selected_path = self.selected_path.as_ref().map(remap);
        if let Some(root) = &self.workspace_root
            && let Some(mapped) = remap_prefix(root, old, new)
        {
            self.workspace_root = Some(mapped);
        }

        self.save_session();
        cx.notify();
    }

    pub(super) fn delete_entry(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = file_name(&path);
        let entity = cx.entity();

        window.open_alert_dialog(cx, move |alert, _window, _cx| {
            let entity = entity.clone();
            let path = path.clone();
            let name = name.clone();
            alert
                .confirm()
                .title("Delete")
                .description(format!("Delete \"{name}\"? This cannot be undone."))
                .ok_text("Delete")
                .ok_variant(ButtonVariant::Danger)
                .cancel_text("Cancel")
                .on_ok(move |_, window, cx| {
                    entity.update(cx, |this, cx| this.remove_entry(&path, window, cx));
                    true
                })
        });
    }

    fn remove_entry(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let ids: Vec<EntityId> = self
            .tabs
            .iter()
            .filter(|tab| {
                tab.path
                    .as_deref()
                    .is_some_and(|candidate| candidate == path || candidate.starts_with(path))
            })
            .map(|tab| tab.editor.entity_id())
            .collect();
        for id in ids {
            self.remove_tab(id, window, cx);
        }

        let result = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        match result {
            Ok(()) => {
                let name = file_name(path);
                self.expanded
                    .retain(|entry| entry.as_path() != path && !entry.starts_with(path));
                if self
                    .selected_path
                    .as_deref()
                    .is_some_and(|selected| selected == path || selected.starts_with(path))
                {
                    self.selected_path = None;
                }
                self.save_session();
                cx.notify();
                window.push_notification(Notification::success(format!("Deleted {name}")), cx);
            }
            Err(error) => window.push_notification(
                Notification::error(format!("Could not delete {}: {error}", file_name(path))),
                cx,
            ),
        }
    }

    pub(super) fn set_file_clipboard(&mut self, path: PathBuf, cut: bool, cx: &mut Context<Self>) {
        self.file_clipboard = Some(FileClipboard { path, cut });
        cx.notify();
    }

    pub(super) fn paste_clipboard(
        &mut self,
        target_dir: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(clipboard) = self.file_clipboard.clone() else {
            return;
        };
        let source = clipboard.path.clone();
        let destination = target_dir.join(file_name(&source));

        if source == destination {
            return;
        }
        if destination.exists() {
            window.push_notification(
                Notification::error(format!("\"{}\" already exists", file_name(&destination))),
                cx,
            );
            return;
        }
        if source.is_dir() && target_dir.starts_with(&source) {
            window.push_notification(
                Notification::error("Cannot paste a folder into itself".to_string()),
                cx,
            );
            return;
        }

        let result = if clipboard.cut {
            std::fs::rename(&source, &destination)
        } else {
            copy_entry(&source, &destination)
        };

        match result {
            Ok(()) => {
                let name = file_name(&destination);
                if clipboard.cut {
                    self.file_clipboard = None;
                    self.after_rename(&source, &destination, cx);
                    window.push_notification(Notification::success(format!("Moved {name}")), cx);
                } else {
                    self.expanded.insert(target_dir);
                    self.selected_path = Some(destination.clone());
                    self.save_session();
                    cx.notify();
                    window.push_notification(Notification::success(format!("Pasted {name}")), cx);
                }
            }
            Err(error) => window
                .push_notification(Notification::error(format!("Could not paste: {error}")), cx),
        }
    }
}
