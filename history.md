# History

All notable changes, decisions, and build-pipeline findings for the Powerapp Frontline Workspace
Shell template. Newest first.

## 2026-10-25 — Pages front door and README published

### Shipped

- Root `index.html` is now the published front door. It follows the sister landing-page pattern
  (max-width wrap, badge, hero, card grid, numbered steps, callout, footer) and uses this product's
  navy/cyan palette rather than importing the sister's emerald theme.
- The page explains the copy → edit `config.json` → commit → Actions artifact flow, links to the
  template's latest build, and names the default-config caveat: the public build is not the maker's
  configuration.
- **No config generator.** The earlier plan for a JSON-generating widget was dropped. Copying the
  template is the intended population method, the repo already ships a complete `config.json`, and the
  page is a front door, not a wizard. This keeps the site one static file with no JavaScript.
- `README.md` is now the terse quickstart: use the template, edit the config, commit, download the
  artifact, with `workflow_dispatch` as the fallback and links to `agents.md`/`history.md`.

### Verified

- Local Chromium at 1280 px and 390 px: title and all four section headings render; the 14 links
  are present and every GitHub destination returns 200; the embedded JSON example parses; no console
  errors; no page-level horizontal overflow.
  The mobile code block scrolls inside its own column after adding `min-width: 0` to the split's grid
  children, which fixed a real page-overflow bug.
- Pages build `40197215452` succeeded (legacy `main` / root, Jekyll, 10s deploy). No Pages workflow
  was added; the branch source picked up the new root `index.html` automatically.
- Live `https://spuds0588.github.io/Powerapp-Frontline-Workspace-Shell/` returns 200, serves the new
  title and H1, and no longer contains the README's "This site is open source" marker. Favicon is 200.
- The release build for the same commit (`40197215725`) is green, so the site commit did not disturb
  the installer pipeline.

### Final state

- Repo remains flagged as a template; Actions is enabled and Pages is built from legacy `main` / root.
- The workspace is 304 KB excluding `.git`; `src-tauri/target` and `src-tauri/gen` are removed.
- Windows install, SSO, kiosk, and memory checks remain documented as manual in `todo.md`; they cannot
  be exercised on this Linux host.

## 2026-10-25 — Windows build action proven end to end

**Status:** `release.yml` is green on both `push` and `workflow_dispatch`. A fresh copy builds in
6m17s cold and 2m27s warm. Both installers were downloaded and inspected.

### Observed runs

| Run | Trigger | Result | Job wall | Build step | Cache |
| --- | --- | --- | --- | --- | --- |
| 40194743841 | push | failure | 1m28s | 1s | none |
| 40194873593 | push | success | 5m43s | 284s | 88 MB (poisoned) |
| 40195334411 | dispatch | success | 5m51s | 288s | 88 MB (poisoned) |
| 40195872005 | push | success | 6m17s | 292s | cold `v1`; saved 372 MB |
| 40196374924 | dispatch | success | 2m27s | 92s | 372 MB restored |

Warm-run per-step wall time (40196374924): setup 1s, checkout 6s, Node 8s, Rust 12s, cache restore
20s, CLI install 2s, build 92s, list 1s, upload 1s, post-cache 0s, post-checkout 2s. Total 147s.

### Artifacts

- `FrontlineShell_1.0.0_x64_en-US.msi` — 2,985,984 bytes, WiX 3.14.1 MSI, SHA-256
  `5a7384fe4bf698a2a26059b21467a0c37608e707d45075013aace6174dcbced7`.
- `FrontlineShell_1.0.0_x64-setup.exe` — 1,944,832 bytes, NSIS 3.13 self-extracting PE32, SHA-256
  `88b67f43e3b8a07bae532ebeaf734f91dcb214e0b8f479a7fb915b7c1b2f87fd`.
- Uploaded artifact `FrontlineShell-Windows-Installer.zip` — 4,702,521 bytes, still unexpired.
- Hashes change run to run (embedded timestamps), so they are not a reproducibility check.

**Landing locations.** CI checks out at `D:\a\<repo>\<repo>` and writes bundles to
`src-tauri/target/release/bundle/msi/` and `.../bundle/nsis/`. `gh run download` preserves the
`msi/` and `nsis/` subdirectories under the destination. The repo-root globs above are correct for a
copied template because the checkout directory name changes with the repo.

### Findings for the next Tauri template

1. **PowerShell breaks a comma-separated `--bundles`.** `tauri build --ci --bundles msi,nsis` failed
   in 1 second with `invalid value 'msi nsis'`; the comma became a space inside one argument. Because
   `tauri.conf.json` already sets `bundle.targets` to both, the fix is to omit the flag entirely.
   Use the config as the single source of truth.
2. **`cache-on-failure` can poison a stable cache key.** The first run failed before compiling but
   saved an 88 MB cache. Every later run restored it, reported `Cache up-to-date`, and never saved the
   real target, so dependencies recompiled every time (284–292s build). Removing `cache-on-failure`
   and bumping `prefix-key` to `v1` produced a 372 MB cache; build fell to 92s and the job to 2m27s.
   Never let failed runs write a key that successful runs reuse.
3. **Prebuilt Tauri CLI is fast.** `npm install -g @tauri-apps/cli@2.12.4` took 2–3s, versus compiling
   `tauri-cli` with `cargo install`. npm had 2.12.4 while the Rust crate resolved 2.12.5; the patch
   mismatch is harmless. Pin both, and expect npm to lag crates.io by a patch.
4. **Pin Rust 1.95.0.** The global-shortcut plugin needs Rust 1.90+, and 1.95.0 is the version the
   local `cargo check` used. The Windows runner also had 1.99.1 installed; `rust-cache` hashes the
   installed Rust environment into its key, which is fine as long as setup stays pinned.
5. **Tauri fetches bundler tooling on every run.** NSIS 3.13 plus `nsis_tauri_utils` 0.5.3 (~4s) and
   WiX 3.14.1 (~1s) are downloaded outside `rust-cache`. They are not worth caching yet; budget
   roughly 6s if adding more bundle targets.
6. **Both bundles need no manual toolchain setup** on `windows-latest`. NSIS is produced first, then
   WiX/MSI. Installers are unsigned, so SmartScreen warnings are expected until a certificate is
   configured. The MSI filename carries `en-US` and both names carry `x64`.
7. **Node 20 action warning.** `actions/checkout@v4`, `setup-node@v4`, and `upload-artifact@v4` run on
   Node 24 with a deprecation warning. It does not fail; bump majors when the template is next touched.
8. **Template-copy trigger.** There is no repository-created event. Both a normal push and
   `workflow_dispatch` were proven here; the copy's initial commit may not run, so the maker's first
   `config.json` edit is the natural trigger and the manual dispatch is the guaranteed fallback.
   This must be the first thing the README says.

### Not verified

- Installing or launching either installer on Windows, SSO, kiosk behavior, and memory use. Only the
  artifact structure was inspected (`file`), not executed.
- A real repository copied from this template; no extra repo was created just for the test.

## 2026-10-25 — Phase 1–2 implemented and compile-verified

### Built

- `config.json` (34 lines): documented schema, dual mode, right sidebar at 460 px, Teams secondary.
- `.gitignore`: Rust target, generated Tauri schemas, logs; `config.json` stays tracked.
- `ui/index.html` (37 lines): the documented dark splash, kept as the packaged frontend asset.
- `src-tauri/Cargo.toml` and `tauri.conf.json`: Tauri 2, `unstable`, MSI + NSIS, `frontendDist: ../ui`.
- `src-tauri/build.rs` (16 lines) and `src-tauri/src/config.rs` (35 lines): one schema, parsed at build
  time and embedded at runtime.
- `src-tauri/src/main.rs` (198 lines): window, child views, layout, allowlist, context menu, kiosk,
  resize sync, supervisor exit.
- `src-tauri/icons/`: 32 px, 128 px, and a multi-size `.ico` generated locally with Pillow.
- `Cargo.lock` kept for reproducible template builds.

### Verified

- `cargo fmt --check` passes.
- `cargo check` exits 0 on Rust 1.95 with Tauri 2.12.5 after the dependency build (subsequent runs
  under 1 s).
- **FR-01 negative test:** adding a trailing comma to `config.json` made `cargo check` exit 101 with
  `[CONFIG][FATAL] config.json is invalid: key must be a string at line 6 column 25`. The original
  file was restored and rechecked; `python3 -m json.tool` and `cargo check` both pass.
- Not exercised: opening a window, navigation blocking against a live URL, kiosk, the shortcut at
  runtime, SSO, installer creation, or memory use. This Linux host has no Windows GUI or MSI path.

### Deviations from the PRD sample, and why

1. **`tauri-plugin-global-shortcut` added.** Tauri v2 has no core global-shortcut API, and FR-03
   requires `Ctrl+Shift+Q`. Registered Rust-side, so no frontend permission or capability is exposed.
   The official plugin requires Rust 1.90+, which is relevant to CI toolchain choice.
2. **`build.rs` parses the shared `config.rs` schema**, not just JSON syntax. A missing or misspelled
   required key is now a compile error too, which is stronger than the doc's runtime `expect`.
3. **Host comparison lowercases both sides.** DNS hostnames are case-insensitive; the sample's exact
   string compare would reject `Apps.PowerApps.com`.
4. **Layout clamps so the primary pane keeps at least 200 px**, adds `min_inner_size(640, 400)`, and
   caps the sidebar at 75%. The sample's `.min(75%).max(100)` could overflow a narrow window.
5. **Dual mode with no secondary pane falls back to single.** The sample leaves a blank strip where
   the secondary would be.
6. **`withGlobalTauri` omitted** (defaults false). Nothing in the local page uses Tauri IPC, and the
   locked-down posture is better without exposing it.
7. **`WebviewUrl` is imported from `tauri`, not `tauri::webview`.** On Tauri 2.12.5 the latter is a
   private re-export and fails to compile; this cost one build iteration.
8. **`ui/index.html` is packaged but not mounted.** The doc's architecture is a bare `Window` plus
   two remote child webviews, so no local webview ever displays the splash. Making it visible would
   need an extra webview, contrary to the memory goal.
9. **`src-tauri/gen/schemas/` is git-ignored.** `tauri-build` regenerates it.

### Next

Phase 3: `release.yml`, a real Windows run, and the artifact inspection. Then Pages and README.

## 2026-10-25 — Mission start: doc audit, repo survey, working contract

### What the master dev doc requires

The product is a **Tauri v2 Windows desktop shell** (`PRD-Powerapp Frontline Workplace Shell.md`),
not a web app. Requirements as understood:

- **FR-01 Compile-time config.** Root `config.json` is embedded with `include_str!`; invalid JSON must
  fail the build, not fail at runtime.
- **FR-02 Layout.** Single mode fills the client area with the primary pane. Dual mode splits into
  primary + secondary with left/right docking and a clamped pixel sidebar width; both rectangles are
  recomputed on native resize.
- **FR-03 Security.** Apex-domain allowlist (subdomains included) enforced on every navigation in both
  panes, context-menu suppression, optional borderless kiosk, and a `Ctrl+Shift+Q` supervisor exit.
- **Distribution.** A Windows GitHub Actions job produces MSI and NSIS installers. Makers never install
  Rust or build locally. ~40–80 MB idle RAM is the target.
- **Explicit non-goals.** DOM scraping, script injection into remote pages, IPC state, databases,
  multi-tenant partitioning, updaters.

### What exists in the repo

- `PRD-Powerapp Frontline Workplace Shell.md` — 468 lines. Contains the PRD, implementation guide,
  developer task list, and most of an `AGENTS.md`. **The file is truncated mid-sentence at Part 4 rule
  5** ("using c"); there is no missing tail on disk. Parts 1–3 are intact.
- `README.md` — one paragraph, no quickstart or config reference.
- `LICENSE` — MIT, Corey Burns, 2026.
- No app source, no workflow, no icons, no Pages `index.html`, no `.gitignore`.
- Repo is **already flagged as a template** (`is_template: true`), public, admin access available.
- **Actions enabled** with all actions allowed; only the Pages build workflow currently exists.
- **Pages enabled, legacy branch source, `main` / root**, currently rendering `README.md` at
  `https://spuds0588.github.io/Powerapp-Frontline-Workspace-Shell/` (HTTP 200).
- Local toolchain: Rust 1.95, Node 20, Python 3 + PyYAML/Pillow. WebKitGTK 4.1, GTK3, libsoup3 dev
  packages are present, so `cargo check` is a realistic local gate. No Tauri CLI installed.
  No MSI install/run path (Linux host).

### Sister-project patterns inspected

- `Spuds0588/Powerapps-to-electron-template` — root `index.html` for Pages (initially a redirect to
  `wizard.html`), single-file vanilla wizard, Pages built from `main` branch root, no Pages workflow,
  `config.json` git-ignored because Electron reads it at runtime.
- `Spuds0588/Powerapp-to-Extension-Template` — root `index.html` + `app.js`/`style.css`, Pages serves
  the wizard directly. No backend.
- `Spuds0588/los-companion-electron-template` — root `index.html` landing page, no build step.
- `Spuds0588/CabinetCast` — uses a Pages Actions workflow because its site is built into `dist`; ours
  does not need that.

**Decision:** keep Pages on the existing legacy branch source and ship a root `index.html`. No Pages
deploy workflow, no `_site` assembly. This is the smallest thing that matches the sisters and the
current repo settings.

### Decisions for this build

- Root `index.html` is the template's front door: explain "Use this template → edit `config.json` →
  push → download installer" and provide a dependency-free config generator (copy/download). This is
  the sister-pattern equivalent of their wizard without adding a multi-step app the PRD never asked
  for.
- `config.json` **is tracked**. The sister Electron template ignores it; this template bakes it in at
  compile time, so CI must see it.
- Add `src-tauri/build.rs` (missing from the doc's tree) both for `tauri_build` and to make invalid
  config JSON a compile error, which is what FR-01 actually demands. The doc's sample only fails at
  runtime via `expect`.
- Implement `Ctrl+Shift+Q` in `main.rs`; FR-03 requires it but the doc's sample code never registers it.
- Generate and commit the icon files the doc's `tauri.conf.json` references; without them bundling
  fails.
- Build only Windows targets in CI. Do not add macOS/Linux jobs.

### GitHub Actions findings log

The user asked for reusable findings from getting the build action working. This section grows as the
workflow is exercised; entries are marked **verified** only after an observed run.

1. **Copying a template does not reliably trigger a build.** GitHub has no "repository created from
   template" event. The initial copy commit may not emit a `push` event, and behavior has been
   inconsistent across reports. *Status: confirmed by GitHub community discussion #52965 and Actions
   runner issue #673; still to be reproduced on this repo.* Mitigation: document a config edit commit
   as the trigger and keep `workflow_dispatch` as the guaranteed first run. Do not build a fragile
   "is this the first commit" detector.
2. **`cargo install tauri-cli` compiles the CLI from source** and adds avoidable minutes to every cold
   build. The doc's workflow does this. *Status: expected; measure in CI.* Plan: install
   `@tauri-apps/cli@^2` from npm (prebuilt) or use `tauri-apps/tauri-action`, and cache Rust.
3. **The doc's source tree omits `build.rs` and all icon files**, both of which Tauri needs. *Status:
   confirmed by the doc; will be proven by the first compile/bundle.*
4. **Tauri child webviews require the `unstable` feature** on the `tauri` crate (`WebviewBuilder`,
   `Window::add_child`). *Status: confirmed in Tauri docs.*
5. **Template copies start with Actions enabled but no run history**, so the first failure is often a
   missing icon/`build.rs` or an unpinned toolchain, not a permissions problem. Keep the installer
   upload step failing loudly (`if-no-files-found: error`) so a silent green build is impossible.
6. **MSI bundling on `windows-latest` depends on WiX/NSIS being fetched by the Tauri CLI.** First run
   may be slower; keep a generous job timeout and record the observed duration.
7. **`config.json` must be committed**, unlike the sister Electron repo. Copying that repo's
   `.gitignore` verbatim would break the CI build and the "no loose files" guarantee.

### Open questions

- The PRD says "custom pixel or proportional split boundaries" but the schema only exposes pixel
  `sidebarWidth`. Implement pixel (schema is authoritative) and note the gap.
- The doc's `layout.mode` is a free string; treat anything other than `"dual"` as single, matching the
  sample code.
- Exact Windows installer behavior, SSO popups, kiosk, and RSS < 90 MB cannot be verified on this
  Linux host. These stay manual/handoff checks.
- Whether to point the GitHub repo's homepage at the Pages URL. Not required; decide at launch.

### Next

Phase 1 and 2 from `todo.md`: config, splash, Tauri scaffold, icons, `cargo check`. Then the workflow
and a first real CI run before the Pages page and README rewrite.
