# Agent Guide — Powerapp Frontline Workspace Shell

Authoritative spec: [PRD-Powerapp Frontline Workplace Shell.md](PRD-Powerapp%20Frontline%20Workplace%20Shell.md).
Read it before changing behavior. If code and the PRD disagree, either fix the code or record the
decision in [history.md](history.md).

## What this is

A Tauri v2 Windows desktop shell that wraps Power Apps, Fabric, and adjacent operational web apps in
one locked-down window. `config.json` is embedded into the binary at compile time with
`include_str!`, so the shipped installer needs no loose files. The repo is a GitHub template: a maker
copies it, edits `config.json`, pushes, and GitHub Actions produces the `.msi` / `.exe`.

## Stack and planned layout

| Path | Purpose |
| --- | --- |
| `config.json` | The only file a maker edits. Tracked in git so CI can embed it. |
| `src-tauri/src/main.rs` | Window, child webviews, layout, allowlist, kiosk, exit shortcut. |
| `src-tauri/src/config.rs` | The config schema, shared by `main.rs` and `build.rs` so it cannot drift. |
| `src-tauri/build.rs` | Parses `config.json` at compile time, then runs `tauri_build::build()`. |
| `src-tauri/Cargo.toml` | `tauri` (`unstable` for child webviews), `serde`, `serde_json`, `url`. |
| `src-tauri/tauri.conf.json` | Product metadata, `frontendDist: ../ui`, MSI + NSIS bundles. |
| `ui/index.html` | Packaged dark splash asset. The bare-window shell is covered by its remote child views, so this is not mounted. |
| `index.html` | GitHub Pages front door: explains the template flow and generates `config.json`. |
| `.github/workflows/release.yml` | Windows installer build on push to `main` and manual dispatch. |

## Commands

```bash
cargo check --manifest-path src-tauri/Cargo.toml   # compile gate (Linux WebKit deps are present here)
cargo fmt --manifest-path src-tauri/Cargo.toml      # formatting
python3 -m json.tool config.json                   # config syntax
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml'))"  # workflow syntax
python3 -m http.server 8080                        # inspect the Pages page locally
gh workflow run release.yml && gh run watch        # CI run (after push)
gh run download <run-id> -n FrontlineShell-Windows-Installer
```

The maker never installs Rust. The canonical build is `windows-latest`; local Linux compilation is
only a fast syntax/type gate. There is no npm project and no frontend build step — keep it that way.

## Rules

1. **YAGNI.** No settings UI, IPC, database, auto-updater, plugins, telemetry, or runtime config
   reads. The shell orchestrates windows and enforces the allowlist; nothing else.
2. **One-line solutions.** Prefer a standard-library or idiomatic one-liner. Add a helper only when
   it removes a real duplicate or encodes a rule.
3. **No new crates** beyond the four listed, `tauri-build`, and the official
   `tauri-plugin-global-shortcut`. The plugin is sanctioned because Tauri v2 has no core shortcut API
   and FR-03 requires `Ctrl+Shift+Q`; register it from Rust, never from remote-page JavaScript.
4. **Config stays embedded.** `config.json` is read only by `include_str!` (and by `build.rs` for
   validation). Never add `std::fs` config loading or a user-editable file next to the installer.
5. **Fail closed.** A non-empty `allowedDomains` blocks every host outside the list and its
   subdomains. An empty list means allow-all; document it, don't silently change it.
6. **Diagnostics to stdout** with stable prefixes: `[INIT]`, `[CONFIG]`, `[SECURITY]`, `[LAYOUT]`,
   `[FATAL]`.
7. **Pages page stays one file.** `index.html` must be self-contained, dependency-free, backend-free,
   and work from a branch-source Pages deployment (no Jekyll, no build step).
8. **Don't add a Pages workflow.** Pages is already enabled with the legacy `main` / root source and
   will serve the new root `index.html`. Adding an Actions-based Pages deploy would require switching
   the source and is unnecessary work.
9. **Full files.** When generating or replacing code, write complete runnable files.

## Template-copy flow (the distribution model)

1. **Use this template** on GitHub (the repo is already flagged as a template).
2. Edit `config.json` — the title, pane URLs, allowed domains, and layout.
3. Commit. That push is what starts `release.yml`; a repo copied from a template has no reliable
   "repository created" event, and the initial copy commit may not emit a workflow-triggering push.
   If no run appears, use **Actions → Build Enterprise Desktop Shell → Run workflow**.
4. Download the `FrontlineShell-Windows-Installer` artifact and install.

Never git-ignore `config.json`. (The sister Electron template ignores its config because the app
reads it at runtime; this template embeds it, so it must be committed.)

## Verification gates

| Change | Must run |
| --- | --- |
| Any Rust | `cargo fmt --check`, `cargo check --manifest-path src-tauri/Cargo.toml` |
| `config.json` | `python3 -m json.tool`; a compile must fail on invalid JSON (build.rs enforces it) |
| Workflow | YAML parse, then a real dispatched run; inspect logs, artifact names, and job wall time |
| `index.html` / README | Serve locally, exercise the page in a browser, then check the live Pages URL |
| Shell behavior | Run on Windows and confirm pane layout, blocked navigation log, kiosk, `Ctrl+Shift+Q` |

CI cache keys must only be written by successful builds. `cache-on-failure` once saved an incomplete
target under a reusable key, so every later run reported the cache as current and recompiled the
dependency tree. The workflow uses `prefix-key: v1`; bump it only to invalidate a poisoned cache.

`Ctrl+Shift+Q` is the supervisor exit and must work in kiosk mode. Keep it registered globally on the
root window, not injected into the remote page.

## Cleanup after builds

Remove `src-tauri/target/` after local compile checks; it is hundreds of MB and git-ignored. Do not
leave downloaded CI artifacts, screenshots, or logs in the workspace. Finish with `git status` and a
`du -sh` sanity check.
