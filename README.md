# Tinytext

Tinytext is a native desktop text editor built with [GPUI Kit](https://gpui-kit.com)
(GPUI + GPUI Component) in Rust.

## Features

- Multi-tab editing with independent rope-backed buffers
- Open, edit, and save files asynchronously
- Tree-sitter syntax highlighting for Rust, TOML, JSON, Markdown, JavaScript,
  TypeScript, Python, HTML, CSS, and PHP
- Unsaved-changes confirmation and save notifications
- File explorer sidebar and resizable panes
- Cursor position, encoding, and language mode in the status bar
- Global shortcuts (`Cmd+N`, `Cmd+O`, `Cmd+S`, `Cmd+W`, `Cmd+B`)
- Dark theme by default

## Requirements

- Rust 1.92 or later
- macOS 15 or later with Xcode Command Line Tools (`xcode-select --install`)

## Run

```sh
cargo run
```

## Build a shareable app

```sh
./scripts/bundle.sh
```

This produces `target/release/bundle/osx/Tinytext.app` and
`target/release/bundle/dmg/Tinytext.dmg`, and ad-hoc signs the app so it launches
locally. Sharing it on another Mac without Gatekeeper warnings needs a Developer ID
signature and notarization; the script prints the exact commands.
Regenerate the placeholder icon with `python3 scripts/make-icon.py`.

## Layout

- Top: MenuBar & Tabs
- Middle: File Explorer sidebar | main editor view
- Bottom: Status bar

## Author

Nilambar Sharma — [nilambar.net](https://nilambar.net/) · [GitHub](https://github.com/ernilambar)

