# Contributing to Tinytext

Tinytext is a small, early-stage project, so contributions are welcome from anyone —
bug reports, fixes, documentation, and feature work. This guide covers setting up the
project, making changes, and running the checks required before a pull request.

## Who can contribute

Anyone. The project is maintained by
[Nilambar Sharma](https://nilambar.net/) ([@ernilambar](https://github.com/ernilambar)),
but external contributions are encouraged.
If you are planning a larger change, open an issue to discuss the approach first so we
can agree on the direction before you write code.

## Install

### Requirements

- Rust 1.92 or later — install with [rustup](https://rustup.rs)
- macOS 15 or later
- Xcode Command Line Tools — `xcode-select --install`
- `rustfmt` and `clippy` components — `rustup component add rustfmt clippy`

### Set up

```sh
git clone git@github.com:ernilambar/tinytext.git
cd tinytext
cargo build
```

The first build compiles GPUI and its dependencies and can take several minutes. Later
builds are fast.

## Fix and run

```sh
cargo run
```

This opens the native window. It requires a graphical macOS session; there is no
headless mode yet.

Application code lives in `src/main.rs`. Keep a feature in that file until it grows
large enough to warrant its own module.

## Check locally before opening a pull request

Run these from the repository root and make sure they all pass:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

- `cargo fmt --check` — confirms the code matches `rustfmt` formatting.
- `cargo clippy --all-targets -- -D warnings` — lints the crate and treats warnings as errors.
- `cargo test` — runs the test suite (there are no tests yet; the harness is in place for future work).
- `cargo build` — confirms the app still compiles.

A future-incompatibility warning from a transitive dependency (`block`) is expected and
is not caused by application code.

## Open a pull request

- Branch off `main`, push your branch, and open a pull request against `main`.
- Write a clear, concise commit message and pull request description explaining what
  changed and why.
- Make sure the checks above pass, and mention anything you could not verify locally.

## Resources

- [GPUI Kit documentation](https://gpui-kit.com/docs/)
- [GPUI Kit component catalog](https://gpui-kit.com/component)
