# TODO

Working checklist for the template. Items are checked only when implemented and, where possible,
verified in this repo.

## Phase 1 — Repository shape

- [x] Read the original brief end to end and survey the repo.
- [x] Create `agents.md`, `todo.md`, `history.md`.
- [x] Add root `config.json` using the documented schema (dual mode, right sidebar, Teams example).
- [x] Add `.gitignore` (`src-tauri/target/`, generated schemas, logs); keep `config.json` tracked.
- [x] Add `ui/index.html` splash exactly in the documented low-key style.
- [x] Absorb the product contract into `agents.md` and remove the original brief from the repo.

## Phase 2 — Tauri v2 shell

- [x] `src-tauri/Cargo.toml` with `tauri` (`unstable`), `serde`, `serde_json`, `url`, `tauri-build`,
      plus the official global-shortcut plugin required for the exit shortcut.
- [x] `src-tauri/build.rs` and shared `src-tauri/src/config.rs`: parse `config.json` so invalid JSON
      fails the compile, then `tauri_build::build()`.
- [x] `src-tauri/tauri.conf.json`: product name, identifier, `frontendDist: ../ui`, MSI + NSIS.
- [x] Generate committed icons (`32x32.png`, `128x128.png`, `icon.ico`) referenced by the bundle.
- [x] `src-tauri/src/main.rs`:
  - [x] `include_str!` config and deserialize.
  - [x] Single/dual child webviews with left/right docking and clamped sidebar width.
  - [x] Recompute both view rectangles on `WindowEvent::Resized`.
  - [x] `on_navigation` allowlist check on both panes with `[SECURITY][BLOCKED]` logging.
  - [x] Context menu suppression via initialization script.
  - [x] Kiosk / fullscreen window flags.
  - [x] `Ctrl+Shift+Q` supervisor exit, registered outside the remote page.
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

- [x] Root `index.html`, single file, zero dependencies: what it is, "Use this template", the
      config-edit → push → artifact flow, and links to the latest build and source.
- [x] Match the sister projects' published-site pattern (root `index.html`, no backend); use the
      established blue/ink palette, not a new design system.
- [x] Replace the abstract window mock with the real Power Apps / Teams reference preview.
- [x] Add the sister-template agent handoff prompt: copy, configure, commit, build.
- [x] Verify locally in Chromium: no console errors, no page overflow at 1280 px or 390 px.
- [x] Push, confirm Pages rebuilt, and check the live URL renders (not README).
- [x] Rewrite `README.md`: quickstart, config reference, CI + template-copy flow, maintainer links.

## Phase 5 — Handoff

- [x] Repo remains flagged as a template; Actions and Pages enabled.
- [x] Record unverifiable checks as manual: install the MSI on clean Windows, confirm
      zero-dependency operation, measure RSS < 90 MB, confirm blocked-navigation behavior on Windows.
- [x] Clean `src-tauri/target/`, CI artifacts, screenshots, logs; `git status` clean except intended
      commits.

## Phase 6 — Demo profile and CI build test

- [x] Add static `demo/` pages and `config.demo.json`.
- [x] Add a `config` choice (`production` / `demo`) to `workflow_dispatch`; demo copies the test
      profile into `config.json` before the build.
- [x] Replace the synthetic dispatch board/chat with labeled Microsoft Learn references: a Power Apps
      model-driven dashboard and a Teams recap.
- [x] Rebuild the separate demo page around the real panes, three layout examples, the embedded
      config, and a link to the prebuilt demo installer (later merged into root `index.html`).
- [x] Update README, `agents.md`, and the Pages front door for the demo flow.
- [x] Confirm Pages publishes `/demo/` and both pane pages return 200.
- [x] Dispatch `config: demo`, watch the Windows run, and fix any failure.
- [x] Download the demo artifact and confirm both installers exist; verify the embedded title and URLs
      in the extracted app binary, not just the artifact name.
- [x] Load the live demo pages in the shell on this Linux host; both navigations are allowed. The
      allowlist block was verified in the earlier local playtest and the code is unchanged (GTK
      stacking remains the known limitation).
- [x] Record the demo run and findings in `history.md`.
- [x] Publish the new demo build as a `demo-latest` release and verify both assets download.

## Phase 7 — Future state (research complete, implementation deferred)

- [x] Research Linux and macOS build requirements, runner labels, bundle targets, macOS
      signing/notarization, Linux packaging, and the product-level blockers. Recorded in
      `history.md`.
- [ ] Implement a per-OS build matrix only after a product decision to support Linux/macOS.
- [ ] Add `tauri.linux.conf.json` / `tauri.macos.conf.json`, `icon.icns`, and `128x128@2x.png`.
- [ ] Fix or work around Linux child-webview stacking and Wayland global shortcuts before shipping
      a Linux kiosk build.
- [ ] Add macOS Developer ID signing/notarization and a DMG fallback before distributing a macOS
      build.

## Phase 8 — Landing page as the demo

- [x] Merge the separate demo page into the root `index.html`; delete `demo/index.html`.
- [x] Replace the three same-app layout thumbnails with three real pairings on a simulated Windows
      desktop: Power Apps + Teams, Power Apps + Teams Phone, and Dynamics 365 + Power Apps.
- [x] Add labeled Microsoft Learn screenshots for Teams Phone and Dynamics 365, plus a Power Apps
      canvas app crop. Keep the existing Power Apps and Teams references.
- [x] Add a three-scene switcher and right/left/single rail control; keep the page static apart from
      the small inline script.
- [x] Update README, `agents.md`, and attribution for the root-page demo.
- [x] Verify in Chromium at 1280 and 390: every scene, every rail, no overflow, no console errors.
- [ ] Push, confirm Pages rebuilds, and check the live root page.

## Playtest evidence

- [x] Run the debug shell on X11 against local pages: window creation, both pane loads, allowlist
      block, context-menu suppression, resize event, and `Ctrl+Shift+Q` exit all observed. Tauri's GTK
      backend stacks child webviews, so dual-pane placement is not verifiable here.

## Manual / cannot verify in this Linux environment

- [ ] Install and launch the MSI on Windows 10/11.
- [ ] Launch the demo-profile installer on Windows and confirm dual-pane placement.
- [ ] Memory measurement in Task Manager under active Power Apps load.
- [ ] Microsoft Entra ID SSO popup behavior against a live tenant.
- [ ] Kiosk behavior on a Windows kiosk account.
