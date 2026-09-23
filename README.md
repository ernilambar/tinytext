# Tinytext

A minimal native macOS text editor prototype built with GPUIX, React, and Bun.

## Run on macOS

Requires [Bun](https://bun.sh/).

```sh
bun install
bun run dev
```

This opens the native app and enables hot reload. Use `bun run start` to launch without hot reload.

## Build macOS apps

Packaging targets **Apple Silicon macOS (arm64)**, matching GPUIX's published native runtime. Install [Rust](https://www.rust-lang.org/tools/install) and the `cargo-packager` CLI:

```sh
cargo install cargo-packager --locked
```

Build a standalone executable with its required native sidecar, or a Finder-launchable `.app`:

```sh
bun run build:bin
./dist/Tinytext
bun run build:app
```

Build a `.dmg` installer (also outputs the `.app`):

```sh
bun run build:dmg
```

Outputs are written to `dist/` and `bundle/`. Launch the app bundle with `open bundle/Tinytext.app`. Builds must run on macOS. These bundles are unsigned; for local testing, use Finder's Open action if Gatekeeper blocks the first launch. Signing and notarization are needed for distribution to other Macs.

## Editing files

Use **Open** or **⌘O** to open a UTF-8 text file. Use **Save** or **⌘S** to save changes; the first save opens the native Save As panel. When opening another file with unsaved changes, Tinytext asks whether to save, discard, or cancel. File access errors appear below the editor.

## Development checks

```sh
bun test
bun run typecheck
```
