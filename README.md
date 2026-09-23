# Tinytext

A minimal native macOS text editor prototype built with GPUIX, React, and Bun.

## Run on macOS

Requires [Bun](https://bun.sh/).

```sh
bun install
bun run dev
```

This opens the native app and enables hot reload. Use `bun run start` to launch without hot reload.

## Check types

```sh
bun run typecheck
```

The editor currently keeps text in memory for the open session; files are not saved yet.
