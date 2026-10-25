# Agent Guide — Powerapp Frontline Workspace Shell

## Product contract

A **Tauri v2 Windows 10/11 desktop shell** for Power Apps, Teams, and adjacent operational web apps.
It must do exactly this:

- Embed the root `config.json` at compile time with `include_str!`; an invalid or incomplete config
  must fail the build, not runtime.
- Open one or two child webviews. In dual mode, place the sidebar left or right at the configured
  pixel width, keep the main pane usable, and recompute both rectangles on resize. In single mode,
  one webview fills the window.
- Check every navigation against `allowedDomains` (apex domains plus their subdomains) and log a
  `[SECURITY][BLOCKED]` line on rejection.
- Suppress the context menu when configured.
- Support optional kiosk (fullscreen, no decorations, no resize) and fullscreen.
- Register `Ctrl+Shift+Q` as the global supervisor exit, including in kiosk mode.
- Do nothing else: no settings UI, IPC, database, updater, telemetry, or runtime config reads.

The original brief has been absorbed into this contract, `src-tauri/src/config.rs`, and `history.md`.
If code and this contract disagree, fix the code or record the decision in `history.md`.

## Stack and layout

| Path | Purpose |
| --- | --- |
| `config.json` | The only file a maker edits. Tracked in git so CI can embed it. |
| `config.demo.json` | Test profile pointing at the published `demo/` pages. CI copies it over `config.json` only for a manual `config: demo` run. |
| `demo/` | The two static reference pages the demo-profile installer loads. The landing page is the browser demo. |
| `demo/shots/` | Five labeled Microsoft Learn reference screenshots. Attribution is in `index.html` and the demo pages; they are not live tenant data. |
| `src-tauri/src/main.rs` | Window, child webviews, layout, allowlist, kiosk, exit shortcut. |
| `src-tauri/src/config.rs` | The config schema, shared by `main.rs` and `build.rs` so it cannot drift. |
| `src-tauri/build.rs` | Parses `config.json` at compile time, then runs `tauri_build::build()`. |
| `src-tauri/Cargo.toml` | `tauri` (`unstable` for child webviews), `serde`, `serde_json`, `url`, and the global-shortcut plugin. |
| `src-tauri/tauri.conf.json` | Product metadata, `frontendDist: ../ui`, MSI + NSIS bundles. |
| `ui/index.html` | Packaged dark splash asset. The bare-window shell is covered by its remote child views, so this is not mounted. |
| `index.html` | Published front door, simulated desktop with three reference pairings, and the copyable agent prompt. |
| `.github/workflows/release.yml` | Windows installer build on push to `main` and manual dispatch. |

## Commands

```bash
cargo check --manifest-path src-tauri/Cargo.toml   # compile gate (Linux WebKit deps are present here)
cargo fmt --manifest-path src-tauri/Cargo.toml      # formatting
python3 -m json.tool config.json                   # production config syntax
python3 -m json.tool config.demo.json              # demo config syntax
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml'))"  # workflow syntax
python3 -m http.server 8080                        # inspect the Pages pages locally
gh workflow run release.yml && gh run watch        # production CI run (after push)
gh workflow run release.yml -f config=demo        # demo-profile CI run
gh run download <run-id> -n FrontlineShell-Windows-Installer
```

The maker never installs Rust. The canonical build is `windows-latest`; local Linux compilation is
only a fast syntax/type gate. There is no npm project and no frontend build step — keep it that way.

## Rules

1. **YAGNI.** No settings UI, IPC, database, auto-updater, plugins beyond global-shortcut, telemetry,
   or runtime config reads. The shell orchestrates windows and enforces the allowlist; nothing else.
2. **One-line solutions.** Prefer a standard-library or idiomatic one-liner. Add a helper only when
   it removes a real duplicate or encodes a rule.
3. **No new crates** beyond the four runtime crates, `tauri-build`, and the official
   `tauri-plugin-global-shortcut`. The plugin is sanctioned because Tauri v2 has no core shortcut API
   and the contract requires `Ctrl+Shift+Q`; register it from Rust, never from remote-page JavaScript.
4. **Config stays embedded.** `config.json` is read only by `include_str!` (and by `build.rs` for
   validation). Never add `std::fs` config loading or a user-editable file next to the installer.
5. **Fail closed.** A non-empty `allowedDomains` blocks every host outside the list and its
   subdomains. An empty list means allow-all; document it, don't silently change it.
6. **Diagnostics to stdout** with stable prefixes: `[INIT]`, `[CONFIG]`, `[SECURITY]`, `[LAYOUT]`,
   `[FATAL]`.
7. **Pages stays static and dependency-free.** `index.html` and `demo/` are plain HTML/CSS plus a
   small inline scene switcher, with no backend, external script, font, or build step. They must
   work from the legacy `main` / root Pages source.
8. **Don't add a Pages workflow.** Pages is already enabled with the legacy `main` / root source.
   Adding an Actions-based Pages deploy would require switching the source and is unnecessary work.
9. **Full files.** When generating or replacing code, write complete runnable files.
10. **Demo references are labeled.** `index.html` and `demo/` use Microsoft Learn screenshots only
    to show the shell and pane proportions. Keep the attribution; never claim they are live tenant
    data.
11. **Windows is the only supported target.** Linux and macOS research is in `history.md`. Do not add
    a build matrix or claim cross-platform support without a product decision and a runtime test.

## Template-copy flow (the distribution model)

1. **Use this template** on GitHub (the repo is already flagged as a template).
2. Edit `config.json` — the title, pane URLs, allowed domains, and layout.
3. Commit. That push is what starts `release.yml`; a repo copied from a template has no reliable
   "repository created" event, and the initial copy commit may not emit a workflow-triggering push.
   If no run appears, use **Actions → Build Enterprise Desktop Shell → Run workflow**.
4. To test without a tenant, dispatch again with `config: demo`. That embeds `config.demo.json`,
   which points at this repo's published demo pages, and produces the same installer artifact.
5. Download the `FrontlineShell-Windows-Installer` artifact and install.

The `demo-latest` GitHub release is a prebuilt demo snapshot for evaluators. It is not the maker's
build and is not updated by every push. A copied repository builds its own artifacts.

The landing page includes a copyable prompt for a coding agent. Keep it aligned with this flow: the
agent should read this file, ask for the real URLs/domains/layout, edit only `config.json`, and help
the maker commit and trigger the build.

Never git-ignore `config.json`. (The sister Electron template ignores its config because the app
reads it at runtime; this template embeds it, so it must be committed.)

## Verification gates

| Change | Must run |
| --- | --- |
| Any Rust | `cargo fmt --check`, `cargo check --manifest-path src-tauri/Cargo.toml` |
| `config.json` | `python3 -m json.tool`; a compile must fail on invalid JSON (`build.rs` enforces it) |
| Workflow | YAML parse, then a real dispatched run; inspect logs, artifact names, and job wall time |
| Workflow `config` input | Dispatch with `demo`; confirm the embedded title is `Frontline Demo Console` and installers exist |
| `demo/` / `config.demo.json` | `python3 -m json.tool`; serve locally; check every page at desktop and mobile widths |
| `index.html` / README | Serve locally, exercise the page in a browser, then check the live Pages URL |
| Shell behavior | Run on Windows and confirm pane layout, blocked navigation log, kiosk, `Ctrl+Shift+Q` |

Linux playtests have a known trap: Tauri/Wry packs child webviews into a `gtk::Box` and ignores
`add_child` bounds, so both panes stack vertically. The product target is Windows/WebView2. Do not add
GTK-specific layout code unless cross-platform becomes a real requirement. The full Linux/macOS
build research, including Wayland shortcut and macOS signing limits, is in `history.md`.

CI cache keys must only be written by successful builds. `cache-on-failure` once saved an incomplete
target under a reusable key, so every later run reported the cache as current and recompiled the
dependency tree. The workflow uses `prefix-key: v1`; bump it only to invalidate a poisoned cache.

`Ctrl+Shift+Q` is the supervisor exit and must work in kiosk mode. Keep it registered globally on the
root window, not injected into the remote page.

## Cleanup after builds

Remove `src-tauri/target/` after local compile checks; it is hundreds of MB and git-ignored. Do not
leave downloaded CI artifacts, test screenshots, or logs in the workspace. Keep the five tracked
`demo/shots/` references. Finish with `git status` and a `du -sh` sanity check.
