# TODO

Working checklist for the template. Mirrors the Developer Task List in
[PRD-Powerapp Frontline Workplace Shell.md](PRD-Powerapp%20Frontline%20Workplace%20Shell.md). Items are
checked only when implemented and, where possible, verified in this repo.

## Phase 1 — Repository shape

- [x] Read the master dev doc end to end and survey the repo.
- [x] Create `agents.md`, `todo.md`, `history.md`.
- [x] Add root `config.json` using the documented schema (dual mode, right sidebar, Teams example).
- [x] Add `.gitignore` (`src-tauri/target/`, generated schemas, logs); keep `config.json` tracked.
- [x] Add `ui/index.html` splash exactly in the documented low-key style.

## Phase 2 — Tauri v2 shell

- [x] `src-tauri/Cargo.toml` with `tauri` (`unstable`), `serde`, `serde_json`, `url`, `tauri-build`,
      plus the official global-shortcut plugin required for the exit shortcut.
- [x] `src-tauri/build.rs` and shared `src-tauri/src/config.rs`: parse `config.json` so invalid JSON
      fails the compile (FR-01), then `tauri_build::build()`.
- [x] `src-tauri/tauri.conf.json`: product name, identifier, `frontendDist: ../ui`, MSI + NSIS.
- [x] Generate committed icons (`32x32.png`, `128x128.png`, `icon.ico`) referenced by the bundle.
- [x] `src-tauri/src/main.rs`:
  - [x] `include_str!` config and deserialize.
  - [x] Single/dual child webviews with left/right docking and clamped sidebar width.
  - [x] Recompute both view rectangles on `WindowEvent::Resized` (FR-02).
  - [x] `on_navigation` allowlist check on both panes with `[SECURITY][BLOCKED]` logging (FR-03).
  - [x] Context menu suppression via initialization script.
  - [x] Kiosk / fullscreen window flags.
  - [x] `Ctrl+Shift+Q` supervisor exit — required by FR-03, missing from the doc's sample code.
- [x] `cargo fmt --check` and `cargo check` pass locally; invalid config JSON fails the build.

## Phase 3 — GitHub Actions installer build

- [x] `.github/workflows/release.yml` on `windows-latest`, triggered by push to `main` and
      `workflow_dispatch`.
- [x] Prebuilt Tauri CLI (npm global install) instead of `cargo install tauri-cli` source build.
- [x] Rust cache, `timeout-minutes`, `concurrency`, pinned action versions.
- [x] Upload `bundle/msi/*.msi` and `bundle/nsis/*.exe` with `if-no-files-found: error`.
- [x] Validate YAML locally, push, run the workflow, and record duration + every failure/fix in
      `history.md` (the user asked for reusable build-action findings).
- [x] Download the artifact and confirm both installer files exist and are non-trivial.
- [x] Confirm the workflow is re-runnable via `workflow_dispatch` and succeeds from a cold cache and
      a warm cache (6m17s cold, 2m27s warm).

## Phase 4 — GitHub Pages front door + README

- [ ] Root `index.html`, single file, zero dependencies: what it is, "Use this template", the
      config-edit → push → artifact flow, and a `config.json` generator with copy/download.
- [ ] Match the sister projects' published-site pattern (root `index.html`, no backend); use the
      established blue/ink palette, not a new design system.
- [ ] Verify locally in a browser: generate a config, assert valid JSON and expected fields, no
      console errors.
- [ ] Push, confirm Pages rebuilt, and check the live URL renders (not README).
- [ ] Rewrite `README.md`: quickstart, config reference, security model, CI + template-copy flow,
      troubleshooting, verification status.

## Phase 5 — Handoff

- [ ] Repo remains flagged as a template; Actions and Pages enabled.
- [ ] Record unverifiable Sprint 3 checks as manual: install the MSI on clean Windows, confirm
      zero-dependency operation, measure RSS < 90 MB, confirm blocked-navigation behavior on Windows.
- [ ] Clean `src-tauri/target/`, CI artifacts, screenshots, logs; `git status` clean except intended
      commits.

## Manual / cannot verify in this Linux environment

- [ ] Install and launch the MSI on Windows 10/11.
- [ ] Memory measurement in Task Manager under active Power Apps load.
- [ ] Microsoft Entra ID SSO popup behavior against a live tenant.
- [ ] Kiosk behavior on a Windows kiosk account.
