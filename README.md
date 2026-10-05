# Tinytext

Tinytext is a native desktop text editor built with [GPUI Kit](https://gpui-kit.com)
(GPUI + GPUI Component) in Rust.

## Features

- Multi-tab editing with independent rope-backed buffers
- Open, edit, and save files asynchronously
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

## Layout

- Top: MenuBar & Tabs
- Middle: File Explorer sidebar | main editor view
- Bottom: Status bar

