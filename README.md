# Slim for OpenIDEA

Syntax highlighting for [Slim](https://slim-template.github.io/) templates in [OpenIDEA](https://github.com/l0ng-ai/openidea), maintained by **faraquet**.

Built and verified against OpenIDEA **0.1.42**, using plugin API **1.5**. The compiled plugin requires no Rust or Ruby installation and requests no special permissions.

## Install

1. Download [`faraquet.slim-0.1.0.oiplugin`](https://github.com/faraquet/openidea-slim/releases/download/v0.1.0/faraquet.slim-0.1.0.oiplugin) from the [releases page](https://github.com/faraquet/openidea-slim/releases).
2. Open **Settings → Plugins** in OpenIDEA.
3. Open the gear menu and choose **Install Plugin from Package File…**.
4. Select the downloaded `.oiplugin` file and click **Install**. The package is unsigned, so OpenIDEA displays a trust notice; it needs no special permissions.
5. Open or reopen a `.slim` file. Highlighting activates automatically.

Use the `.oiplugin` release asset for installation. GitHub's **Source code** archives are for building the plugin yourself.

All filenames ending in `.slim` are recognised, including `.html.slim`, `.html+mobile.slim`, and `.html+desktop.slim`. Colours follow your OpenIDEA theme. The language definition uses two-space indentation and `/` for line comments.

For example, create `example.html.slim`:

```slim
doctype html
main#content.card data-controller="booking"
  / This is a Slim comment
  - if @booking
    h1 Welcome, #{user.name}
    = link_to "View booking", booking_path(@booking), class: :button
```

The plugin distinguishes tags, attributes, comments, Ruby keywords, strings, symbols, and interpolated Ruby. There is no additional configuration to enable.

### Troubleshooting and removal

- If highlighting stays plain, check that **Slim** is enabled under **Settings → Plugins**, reopen the file, and restart OpenIDEA if necessary.
- If a file has an explicit language override, change it to **Slim** under **Settings → File Types**.
- Check the plugin details for its running status and error count. Remove another Slim language plugin if it conflicts with file detection.
- To update, install a newer `.oiplugin` release through the same menu. To remove it, select **Slim → Uninstall** in the plugin settings.

Each release includes a SHA-256 checksum. On macOS, verify a download with `shasum -a 256 -c faraquet.slim-0.1.0.oiplugin.sha256` from the folder containing both files; on Linux use `sha256sum -c` instead.

## Highlighting

- HTML tag names and `.class` / `#id` shorthand.
- Attributes, boolean attributes inside wrappers, and multiline attribute values.
- Ruby control/output lines (`-`, `=`, `==`) and `ruby:` blocks.
- Ruby keywords, variables, constants, symbols, strings, escapes, numbers and common regular expressions.
- Ruby interpolation (`#{...}`), including nested strings and Unicode text.
- Multiline helper calls, nested hashes and arrays.
- Slim comments and indented comment bodies, text blocks, doctypes, block expansion, and whitespace markers.

This is a lexical highlighter, not a complete Slim/Ruby parser. Ruby heredocs and ambiguous Ruby syntax are not fully handled. Non-Ruby filters (such as `javascript:`, `css:`, and `markdown:`) and raw HTML lines use text highlighting with Ruby interpolation; they do not have dedicated embedded-language highlighting. It does not provide completion, diagnostics, navigation, folding, or a structure outline.

## Build

The OpenIDEA SDK is vendored from release `v0.1.42`, commit `e702e4da7b0333e81bc818e26be72db1d157ec5d`. No OpenIDEA checkout is needed. The build was tested with Rust 1.99.1.

Requirements: Rust with the `wasm32-wasip2` target and Python 3. Rust 1.99.1 was used for the release build. The optional backend integration test needs Python 3.11 or newer.

```sh
git clone https://github.com/faraquet/openidea-slim.git
cd openidea-slim
rustup target add wasm32-wasip2
cargo test --locked
sh build.sh
```

`build.sh` requires Python 3 and writes `dist/plugin.wasm` and `faraquet.slim-0.1.0.oiplugin`. The lockfile pins Rust dependencies. The plugin contains no network or filesystem calls and needs no Ruby installation at runtime.

To check token bounds and ordering against a local template directory:

```sh
cargo run --locked --release --example check_templates -- /path/to/app/views
```

To repeat the backend integration test, download the matching `openidea-server` executable from the [official OpenIDEA 0.1.42 release](https://github.com/l0ng-ai/openidea/releases/tag/v0.1.42), then run:

```sh
python3 -m venv .venv
.venv/bin/pip install blake3
.venv/bin/python tests/check_runtime.py /path/to/openidea-server
```

This test uses the built `dist/plugin.wasm`, creates a temporary project and profile, and checks synthetic templates without changing your installed OpenIDEA plugins.

## Verification

- 15 tests cover common Slim constructs, Unicode, incomplete editing states, deep nesting, and 2,000 deterministic randomized inputs.
- Validated non-overlapping, in-bounds UTF-8 token ranges on 2,149 real-world templates. Those templates are not included in this repository.
- Loaded the compiled WebAssembly plugin into an isolated official OpenIDEA 0.1.42 backend; verified language detection and returned highlight categories for all four filename variants above.
- Visually verified highlighting in the desktop editor, including Ruby keywords, method calls, symbols, and strings; the installed plugin reported no errors or crashes.

The plugin source is MIT licensed. The vendored OpenIDEA SDK is Apache-2.0 licensed; see `THIRD_PARTY_NOTICES.txt` and `OPENIDEA-LICENSE`.
