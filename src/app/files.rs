use std::path::PathBuf;

use gpui_kit::component::{WindowExt as _, notification::Notification};
use gpui_kit::*;

use crate::language::{editor_language_id, language_for};
use crate::paths::file_name;
use crate::{NewFile, OpenFile, OpenFolder, SaveFile};

use super::TinytextApp;

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
}
