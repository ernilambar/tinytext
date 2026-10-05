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
- `tinytext file.txt` command-line launcher that opens files in the running window
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

## Open files from the command line

The app can add the `tinytext` command to your `PATH` for you: choose
**Help → Install "tinytext" Command in PATH** in the menu bar. macOS asks for
your password once so the launcher can be written to `/usr/local/bin/tinytext`.

To do it manually instead, bundle and install the app, then put the launcher on
your `PATH`:

```sh
./scripts/bundle.sh
cp -R target/release/bundle/osx/Tinytext.app /Applications/
sudo install -m 755 scripts/tinytext /usr/local/bin/tinytext
```

Now `tinytext` opens files in the app:

```sh
tinytext file.txt
tinytext one.txt two.txt
```

The launcher hands the paths to the running Tinytext instance through
LaunchServices (or launches it on first use), so files open as tabs in the
existing window rather than starting a second copy. Point it at a bundle
elsewhere with `TINYTEXT_APP=/path/to/Tinytext.app tinytext file.txt`.

Because the bundle declares text document types, macOS also lists Tinytext
under Finder's "Open With" for text files once the app has been launched.

## Layout

- Top: MenuBar & Tabs
- Middle: File Explorer sidebar | main editor view
- Bottom: Status bar

## Author

Nilambar Sharma — [nilambar.net](https://nilambar.net/) · [GitHub](https://github.com/ernilambar)

