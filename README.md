# Tinytext

Tinytext is a native desktop text editor built with [GPUI Kit](https://gpui-kit.com)
(GPUI + GPUI Component) in Rust.

## Status

Phase 1 — Project setup and core shell. The app opens a window containing the top-level
layout slots (top bar, sidebar, editor, status bar) as placeholders.

## Requirements

- Rust 1.92 or later
- macOS 15 or later with Xcode Command Line Tools (`xcode-select --install`)

## Run

```sh
cargo run
```

## Layout

- Top: MenuBar & Tabs
- Middle: File Explorer sidebar | main editor view
- Bottom: Status bar

## Roadmap

Phase 1 project setup and shell · Phase 2 component tree · Phase 3 buffer logic ·
Phase 4 syntax highlighting and advanced features.
