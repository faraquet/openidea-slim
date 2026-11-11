# Slim for OpenIDEA

Syntax highlighting and optional Ruby language features for [Slim](https://slim-template.github.io/) templates in [OpenIDEA](https://github.com/l0ng-ai/openidea), maintained by **faraquet**.

Built and verified against OpenIDEA **0.1.42**, using plugin API **1.5**. Highlighting needs no Ruby or Rust installation. Optional navigation, hover, and completion use your Ruby LSP installation with the `ruby-lsp-slim` addon.

## Install

1. Download [`faraquet.slim-0.2.1.oiplugin`](https://github.com/faraquet/openidea-slim/releases/download/v0.2.1/faraquet.slim-0.2.1.oiplugin) from the [releases page](https://github.com/faraquet/openidea-slim/releases).
2. Open **Settings → Plugins** in OpenIDEA.
3. Open the gear menu and choose **Install Plugin from Package File…**.
4. Select the downloaded `.oiplugin` file and click **Install**. The package is unsigned, so OpenIDEA displays a trust notice; allow the optional `process:ruby-lsp` permission to enable Ruby language features. Highlighting works without it.
5. Open or reopen a `.slim` file. Highlighting activates automatically.

Use the `.oiplugin` release asset for installation. GitHub's **Source code** archives are for building the plugin yourself.

All filenames ending in `.slim` are recognised, including `.html.slim`, `.html+mobile.slim`, and `.html+desktop.slim`. Tabs and the project tree show an amber **S** icon for Slim files. Files associated with Slim in project settings (such as Slim-based `*.mjml` templates) inherit the same icon. Colours follow your OpenIDEA theme. The language definition uses two-space indentation and `/` for line comments.

For example, create `example.html.slim`:

```slim
doctype html
main#content.card data-controller="booking"
  / This is a Slim comment
  - if @booking
    h1 Welcome, #{user.name}
    = link_to "View booking", booking_path(@booking), class: :button
```

The plugin distinguishes tags, attributes, comments, Ruby keywords, strings, symbols, and interpolated Ruby. Highlighting activates without additional configuration.

### Troubleshooting and removal

- If highlighting stays plain, check that **Slim** is enabled under **Settings → Plugins**, reopen the file, and restart OpenIDEA if necessary.
- If a file has an explicit language override, change it to **Slim** under **Settings → File Types**.
- Check the plugin details for its running status and error count. Remove another Slim language plugin if it conflicts with file detection.
- To update, install a newer `.oiplugin` release through the same menu. To remove it, select **Slim → Uninstall** in the plugin settings.

Each release includes a SHA-256 checksum. On macOS, verify a download with `shasum -a 256 -c faraquet.slim-0.2.1.oiplugin.sha256` from the folder containing both files; on Linux use `sha256sum -c` instead.

## Highlighting

- HTML tag names and `.class` / `#id` shorthand.
- Attributes, boolean attributes inside wrappers, and multiline attribute values.
- Ruby control/output lines (`-`, `=`, `==`) and `ruby:` blocks.
- Ruby keywords, variables, constants, symbols, strings, escapes, numbers and common regular expressions.
- Ruby interpolation (`#{...}`), including nested strings and Unicode text.
- Multiline helper calls, nested hashes and arrays.
- Slim comments and indented comment bodies, text blocks, doctypes, block expansion, and whitespace markers.

This is a lexical highlighter, not a complete Slim/Ruby parser. Ruby heredocs and ambiguous Ruby syntax are not fully handled. Non-Ruby filters (such as `javascript:`, `css:`, and `markdown:`) and raw HTML lines use text highlighting with Ruby interpolation; they do not have dedicated embedded-language highlighting. Ruby language features come from the optional server integration below. The highlighter itself does not provide folding or a structure outline.

## Ruby language features

The plugin registers Slim documents with OpenIDEA's existing `ruby-lsp` server. The [ruby-lsp-slim addon](https://github.com/afomera/ruby-lsp-slim) extracts Ruby from Slim and supplies it to Ruby LSP for definition lookup, hover, and completion. This also keeps ordinary Ruby files on the same server.

1. Install Ruby LSP in the Ruby environment used by your project: `gem install ruby-lsp`.
2. Add `gem "ruby-lsp-slim", "~> 0.1.0", require: false` to your project's development gem group, then run `bundle install`.
3. Install this plugin and allow its optional `process:ruby-lsp` permission.
4. Restart Ruby LSP or OpenIDEA. Open a Slim file and use **Go to Declaration or Usages**, hover over a Ruby method, or invoke completion.

Ruby LSP must load the addon in its active bundle; installing the gem globally alone is insufficient when Bundler is active. The addon requires Ruby 3 or newer, Ruby LSP 0.26 or newer (below 1.0), and Slim 4 or 5. Definition, hover, and completion were verified with Ruby 4.0.1, Ruby LSP 0.27.0.beta5, and ruby-lsp-slim 0.1.0.

### Editor-only bundle

To keep editor dependencies out of your application's Gemfile, create `.openidea/ruby-lsp/Gemfile`:

```ruby
eval_gemfile(File.expand_path("../../Gemfile", __dir__))

gem "ruby-lsp", require: false, group: :development
gem "ruby-lsp-slim", "~> 0.1.0", require: false, group: :development
```

Include `ruby-lsp-rails` here if you already use it. Preserve any version constraints from your current LSP bundle. Copy the application's `Gemfile.lock` to `.openidea/ruby-lsp/Gemfile.lock` before installing, to preserve application dependency versions; check any relative `PATH` dependencies in the copied lockfile. From your project root, run:

```sh
BUNDLE_GEMFILE="$PWD/.openidea/ruby-lsp/Gemfile" bundle install
```

Add the following to the project's `.openidea/settings.toml`, replacing the example path with your absolute project path:

```toml
[languageServers.ruby-lsp.env]
BUNDLE_GEMFILE = "/absolute/path/to/project/.openidea/ruby-lsp/Gemfile"
RUBYOPT = "-rbundler/setup"
```

If you already configure `RUBYOPT`, retain those flags alongside `-rbundler/setup`. Keep this editor bundle and its local settings out of version control. Restart OpenIDEA after editing this settings file so it reloads the environment. Wait for Ruby LSP indexing to finish before testing navigation; large projects can take several minutes.

### Scope and troubleshooting

- **No language server available:** check the plugin's Ruby LSP permission, server executable, and project trust settings.
- **Ruby works but Slim does not:** check the Ruby LSP log for `[Ruby LSP Slim] Addon v0.1.0 activated`. Ensure the addon belongs to the server's active bundle and restart the server.
- Addon support is experimental. Its Slim-to-Ruby scanner does not cover every Slim construct. Rails macros, dynamically generated helpers, inferred receiver types, and navigation to partials depend on the server/addons; this is not full RubyMine feature parity.
- The addon uses Ruby LSP's ERB document support. Syntax errors and formatting are deliberately suppressed for the extracted Ruby, so this integration is not a Slim validator or formatter.

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

`build.sh` requires Python 3 and writes `dist/plugin.wasm` and `faraquet.slim-0.2.1.oiplugin`. The lockfile pins Rust dependencies. The WebAssembly highlighter contains no network or filesystem calls. Ruby is required only for the optional language server.

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

To also test definition lookup, hover, and completion, install Ruby LSP 0.27.0.beta5 and ruby-lsp-slim 0.1.0 in your active Ruby environment, then add `--with-lsp` to the command above. The test resolves these gems locally and does not install anything.

This test uses the built `dist/plugin.wasm`, creates a temporary project and profile, and checks synthetic templates without changing your installed OpenIDEA plugins.

## Verification

- 15 tests cover common Slim constructs, Unicode, incomplete editing states, deep nesting, and 2,000 deterministic randomized inputs.
- Validated non-overlapping, in-bounds UTF-8 token ranges on 2,149 real-world templates. Those templates are not included in this repository.
- Loaded the compiled WebAssembly plugin into an isolated official OpenIDEA 0.1.42 backend; verified language detection and returned highlight categories for all four filename variants above.
- Verified definition lookup from Slim into a Ruby method, hover signatures, and method completion through the official backend with the Slim addon.
- Verified desktop Go to Declaration from a real Slim helper call into its Ruby method definition.
- Visually verified highlighting in the desktop editor, including Ruby keywords, method calls, symbols, and strings; the installed plugin reported no errors or crashes.

The plugin source is MIT licensed. The vendored OpenIDEA SDK is Apache-2.0 licensed; see `THIRD_PARTY_NOTICES.txt` and `OPENIDEA-LICENSE`.
