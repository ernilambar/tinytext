//! Every key binding in the app.
//!
//! Bindings are split in two groups. The app bindings are global and match
//! regardless of focus. The editor bindings use gpui-kit's `"Input"` key
//! context, so they only fire while a text editor has focus. The editor
//! bindings are registered after `gpui_kit::init` has installed the engine's
//! own defaults; at equal context depth the later registration wins, so these
//! take precedence where they overlap with the engine.

use gpui_kit::*;

use crate::{
    CloseTab, CopyLineDown, CopyLineUp, DeleteLine, GoToLine, Hide, HideOthers, InsertLineAbove,
    InsertLineBelow, JumpToTab, MoveLineDown, MoveLineUp, NewFile, NextTab, OpenFile, OpenFolder,
    OpenSettings, PreviousTab, Quit, ReopenClosedTab, SaveAll, SaveFile, SaveFileAs, SelectLine,
    ToggleLineComment, ToggleSidebar, ToggleWordWrap, ZoomIn, ZoomOut, ZoomReset,
};

/// gpui-kit's editor key context. Its `CONTEXT` constant is private, so the
/// literal it uses is repeated here.
const EDITOR: &str = "Input";

/// Registers every binding. Call once during startup, after `gpui_kit::init`.
pub(crate) fn bind_keys(cx: &mut App) {
    cx.bind_keys(app_bindings());
    cx.bind_keys(editor_bindings());
}

/// Bindings that are active regardless of which element has focus.
fn app_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("cmd-n", NewFile, None),
        KeyBinding::new("cmd-o", OpenFile, None),
        KeyBinding::new("cmd-shift-o", OpenFolder, None),
        KeyBinding::new("cmd-s", SaveFile, None),
        KeyBinding::new("cmd-shift-s", SaveFileAs, None),
        KeyBinding::new("alt-cmd-s", SaveAll, None),
        KeyBinding::new("cmd-w", CloseTab, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("alt-cmd-h", HideOthers, None),
        KeyBinding::new("cmd-b", ToggleSidebar, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-=", ZoomIn, None),
        KeyBinding::new("cmd-shift-=", ZoomIn, None),
        KeyBinding::new("cmd--", ZoomOut, None),
        KeyBinding::new("cmd-0", ZoomReset, None),
        KeyBinding::new("cmd-shift-t", ReopenClosedTab, None),
        KeyBinding::new("cmd-shift-]", NextTab, None),
        KeyBinding::new("cmd-shift-[", PreviousTab, None),
        KeyBinding::new("cmd-1", JumpToTab(1), None),
        KeyBinding::new("cmd-2", JumpToTab(2), None),
        KeyBinding::new("cmd-3", JumpToTab(3), None),
        KeyBinding::new("cmd-4", JumpToTab(4), None),
        KeyBinding::new("cmd-5", JumpToTab(5), None),
        KeyBinding::new("cmd-6", JumpToTab(6), None),
        KeyBinding::new("cmd-7", JumpToTab(7), None),
        KeyBinding::new("cmd-8", JumpToTab(8), None),
        KeyBinding::new("cmd-9", JumpToTab(9), None),
    ]
}

/// VS Code's macOS editing chords, scoped to the focused editor.
fn editor_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("cmd-/", ToggleLineComment, Some(EDITOR)),
        KeyBinding::new("cmd-shift-k", DeleteLine, Some(EDITOR)),
        KeyBinding::new("cmd-l", SelectLine, Some(EDITOR)),
        KeyBinding::new("alt-up", MoveLineUp, Some(EDITOR)),
        KeyBinding::new("alt-down", MoveLineDown, Some(EDITOR)),
        KeyBinding::new("shift-alt-up", CopyLineUp, Some(EDITOR)),
        KeyBinding::new("shift-alt-down", CopyLineDown, Some(EDITOR)),
        KeyBinding::new("cmd-enter", InsertLineBelow, Some(EDITOR)),
        KeyBinding::new("cmd-shift-enter", InsertLineAbove, Some(EDITOR)),
        KeyBinding::new("cmd-g", GoToLine, Some(EDITOR)),
        KeyBinding::new("alt-z", ToggleWordWrap, Some(EDITOR)),
    ]
}
