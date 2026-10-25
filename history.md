# History

All notable changes, decisions, and build-pipeline findings for the Powerapp Frontline Workspace
Shell template. Newest first.

## 2026-10-25 — Future state: Linux and macOS builds (research only)

**Verdict:** the build pipeline can produce Linux and macOS installers from native GitHub runners, but
the product cannot be called cross-platform yet. Windows remains the only supported target. No workflow
or bundle change was made; this is the reusable research for a future decision.

### What a three-OS build would take

Use a native matrix, not cross-compilation. Tauri needs the host OS SDK and WebView engine, and macOS
cannot be legally built from Linux. A minimal matrix is `windows-2025`, `ubuntu-24.04`, and
`macos-15` (Apple Silicon). Add `macos-15-intel` only if Intel Macs are a real requirement. Pin labels
rather than `*-latest`: GitHub is retiring `ubuntu-22.04` on 17 April 2027 and has already retired
`macos-13`. `windows-latest` still works today, but pinning `windows-2025` makes the Windows baseline
explicit.

- **Windows:** unchanged. `tauri build --ci` with `bundle.targets: ["msi", "nsis"]` and the current
  artifact upload. Keep the existing cache and timeout rules.
- **Linux:** install `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev` (or
  `libappindicator3-dev` where that is the available name), `librsvg2-dev`, `libxdo-dev`,
  `libssl-dev`, `build-essential`, `patchelf`, and `xdg-utils`. Bundle `deb`, `rpm`, and `appimage`
  through `tauri.linux.conf.json`. Tauri v2 needs WebKitGTK **4.1**, not the old 4.0 package.
  Tauri recommends building on the oldest distro that provides 4.1 because the build host sets the
  glibc floor. Ubuntu 22.04 is that old baseline, but it is being retired from Actions; after April
  2027 either build inside a 22.04 container or accept a newer glibc floor from 24.04.
- **macOS:** bundle `app` and `dmg` through `tauri.macos.conf.json`. A `.icns` icon is required and is
  missing from this repo; `tauri icon` can generate the full set from a square PNG. For one universal
  download, install both Rust targets on an Apple Silicon runner and build
  `--target universal-apple-darwin`; do not try to install `universal-apple-darwin` as a Rust target.
  Separate arm64 and x64 builds are simpler but produce two artifacts.
- **Per-OS config:** Tauri automatically merges `tauri.windows.conf.json`, `tauri.linux.conf.json`,
  and `tauri.macos.conf.json` into `tauri.conf.json`. That is the clean way to keep Windows-only
  `msi`/`nsis` targets from breaking a Linux or macOS build.
- **Artifacts:** rename per OS (`FrontlineShell-Windows-Installer`, `-Linux-Installer`,
  `-macOS-Installer`) and upload only that OS's bundle paths with `if-no-files-found: error`. The
  current fixed artifact name would collide in a matrix.

### macOS signing is the real gate

An unsigned `.dmg` from GitHub is quarantined like any internet download. Users get “cannot verify the
developer” or “cannot check for malicious software”; macOS Sequoia removed the Control-click override,
so they must use **System Settings → Privacy & Security → Open Anyway**. That is not a clean install.

Public Developer ID distribution needs the paid Apple Developer Program (US$99/year), a **Developer ID
Application** certificate, and notarization. Tauri’s CI path is:

- Base64 `.p12` in `APPLE_CERTIFICATE`, its export password in `APPLE_CERTIFICATE_PASSWORD`, the
  identity in `APPLE_SIGNING_IDENTITY`, and a temporary keychain password in `KEYCHAIN_PASSWORD`.
- Notarization credentials: either `APPLE_ID` + `APPLE_PASSWORD` (app-specific password) +
  `APPLE_TEAM_ID`, or `APPLE_API_KEY` + `APPLE_API_ISSUER` + `APPLE_API_KEY_PATH` for the App Store
  Connect key.
- Import the certificate into a throwaway keychain with `security create-keychain`, `security import`,
  and `security set-key-partition-list`; delete it in a `trap` on exit.

Hardened runtime is already the Tauri default and is required for notarization. A normal unsandboxed
WKWebView app does not need a special network entitlement just to load HTTPS; a sandboxed build would
need `com.apple.security.network.client`. Do not add JIT exceptions unless a signed build proves they
are needed.

DMG creation on GitHub is the flakiest part: Tauri’s DMG layout uses Finder/AppleScript, and `CI=true`
passes `--skip-jenkins` to skip that step. `hdiutil: create failed - Resource busy` has appeared
intermittently on hosted runners. The practical fallback is `--bundles app`, then zip the `.app` (or
retry the DMG on another runner), rather than killing XProtect or granting broad privileges.

### Linux packaging caveats

Tauri CLI 2.8+ removed the build-time FUSE requirement from its AppImage tooling, but **running** an
AppImage may still need `libfuse2` (Ubuntu 22.04) or `libfuse2t64` (24.04), or
`--appimage-extract-and-run`. For build failures, `APPIMAGE_EXTRACT_AND_RUN=1` is the no-FUSE
fallback; the `unknown type [0x13] section .relr.dyn` strip failure has a separate `NO_STRIP=true`
workaround. Do not treat either as a universal fix. Media playback needs GStreamer, and
`bundleMediaFramework` only bundles it on Ubuntu-built AppImages; it is not a Widevine guarantee.

### What blocks this product, beyond CI

1. **The product contract is Windows 10/11.** Shipping Linux or macOS is a product decision, not just
   a matrix entry. The kiosk and hardware assumptions in the original spec are Windows-shaped.
2. **Linux dual-pane is still broken upstream.** Tauri places child webviews in a GTK `Box`, so both
   panes stack; the bounds are ignored. Tauri issue #10420 is open. A Linux build would need a
   single-pane mode, a GTK `Fixed` workaround maintained in this repo, or a different window strategy.
   A green Linux build would not make the current dual-pane promise true.
3. **Wayland has no reliable global shortcut.** Tauri’s global-hotkey backend supports X11 only; the
   XDG GlobalShortcuts portal exists, but the plugin does not use it. `Ctrl+Shift+Q` is the kiosk
   escape hatch, so shipping Linux without a working exit is dangerous. X11-only is an option, but
   many current desktops default to Wayland.
4. **macOS kiosk is not Windows kiosk.** Fullscreen plus `decorations(false)` is presentation, not
   OS lockdown. macOS still exposes the menu bar, Dock, app switching, `Cmd+Q`, and fullscreen escape
   gestures. Real single-app mode needs MDM (Autonomous Single App Mode) and device management. Test
   whether the global shortcut is delivered in native fullscreen before relying on it as the only exit.
5. **WKWebView may not be enough for Power Apps SSO.** WebKit’s ITP blocks third-party cookies by
   default; Microsoft documents third-party-cookie failures for embedded Power Apps and recommends
   interactive popup/full-page fallbacks. `WKWebView` also does not share Safari’s cookie jar. This
   needs a real tenant test, not a compile check.
6. **Assets and metadata are Windows-only.** The repo has `32x32.png`, `128x128.png`, and `icon.ico`,
   but no `icon.icns` or `128x128@2x.png`. `tauri.conf.json` hard-codes `msi`/`nsis` targets.
7. **Signing is not solved on any platform.** Windows MSI/NSIS are currently unsigned, so SmartScreen
   will warn. macOS public distribution needs the Apple setup above. Linux packages are unsigned.

### If this is greenlit later

Do it in this order: add the per-OS configs and icons; add a matrix that keeps the Windows job
unchanged; make Linux single-pane or fix GTK placement; add macOS signing/notarization and a DMG
fallback; then test on real Windows, Linux X11, Linux Wayland, and macOS. CI green is not runtime
proof. The Windows path should stay the required gate until the others have an owner and a support
statement.

Sources: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), [Tauri GitHub pipeline](https://v2.tauri.app/distribute/pipelines/github/), [Tauri config reference](https://v2.tauri.app/reference/config/), [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/), [Tauri environment variables](https://v2.tauri.app/reference/environment-variables/), [Tauri AppImage](https://v2.tauri.app/distribute/appimage/), [Tauri CLI target reference](https://v2.tauri.app/reference/cli/), [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners), [Ubuntu 22.04 retirement](https://github.com/actions/runner-images/issues/14254), [Tauri issue #10420](https://github.com/tauri-apps/tauri/issues/10420), [Wayland shortcut issue](https://github.com/tauri-apps/tauri/issues/3578), [Apple Gatekeeper](https://support.apple.com/en-us/102445), [Apple Sequoia change](https://developer.apple.com/news/?id=saqachfa), [GitHub certificate install](https://docs.github.com/en/actions/how-tos/deploy/deploy-to-third-party-platforms/sign-xcode-applications), [Tauri DMG CI issue](https://github.com/tauri-apps/tauri/issues/592), [WKWebView ITP](https://webkit.org/tracking-prevention/), [Power Apps cookie troubleshooting](https://learn.microsoft.com/en-us/troubleshoot/power-platform/power-apps/sign-in/troubleshooting-startup-issues).

## 2026-10-25 — Demo profile and tenant-free CI build

### Shipped

- `demo/` is a static placeholder for a frontline Power App: a dispatch board (`primary.html`), a
  dispatch chat (`secondary.html`), a shared stylesheet, and a framed preview at `demo/` that shows
  both panes side by side. No JavaScript, no dependencies, no build step.
- `config.demo.json` embeds the demo profile: title `Frontline Demo Console`, 1440x900, 420 px right
  sidebar, and `allowedDomains: ["spuds0588.github.io"]` with both pane URLs on the published Pages
  site. It is not the default; `config.json` still points at Power Apps and Teams.
- `.github/workflows/release.yml` now has a `workflow_dispatch` input, `config`, with `production`
  (default) and `demo`. The `demo` step copies `config.demo.json` over `config.json` after checkout
  and before the build. A push always builds production.
- README, `agents.md`, `todo.md`, and the Pages front door now explain the demo and the manual run.

### Verified

- Local Python parse of all demo HTML, `config.demo.json`, and the workflow YAML; served the root and
  all demo assets over HTTP (all 200).
- Chromium at 1280 px and 390 px: both iframes load, frame titles are correct, no console errors, no
  page-level horizontal overflow. The demo preview scrolls each pane independently.
- Pages build `40216024166` succeeded in 1m07s from the push of `97b9a89`. Live URLs:
  `/`, `/demo/`, `/demo/primary.html`, `/demo/secondary.html`, `/demo/demo.css`, and
  `/config.demo.json` all return 200. Jekyll served the subfolder without a `.nojekyll` or config
  change.
- Production build `40216024538` succeeded: 2m23s job, 89s build step, 372 MB cache restored. The
  `Use demo configuration` step was correctly skipped on the push.
- Demo build `40216270479` succeeded on the first attempt: 2m17s job, 84s build step, same 372 MB
  cache. The `Use demo configuration` step ran and the artifact was `FrontlineShell-Windows-Installer`.
- Downloaded the demo artifact: `FrontlineShell_1.0.0_x64_en-US.msi` (2,985,984 bytes) and
  `FrontlineShell_1.0.0_x64-setup.exe` (1,946,048 bytes). Extracted the NSIS installer with `7z`; the
  embedded `frontline-desktop-shell.exe` (8,927,232 bytes) contains
  `"title": "Frontline Demo Console"` and both `spuds0588.github.io/.../demo/*.html` URLs, and no
  `Frontline Operations Console` string. This proves the selector embedded the demo profile rather
  than just naming the run.
- Built the Linux debug binary with the demo profile and ran it on X11: `[CONFIG][EMBEDDED] Loaded
  profile: 'Frontline Demo Console' | Mode='dual'`, `[LAYOUT][SYNC] ... 1440x900`, and both
  `[NAVIGATION][ALLOWED]` lines for the live demo pages. The process stayed alive until the 25s
  timeout. GTK still stacks the child views; that is the known Linux backend limitation, not a demo
  problem. The allowlist block was not repeated this pass; the earlier local redirect test and the
  unchanged `navigation_guard` code cover it.

### Timings

| Run | Trigger | Result | Job | Build step | Cache |
| --- | --- | --- | --- | --- | --- |
| 40216024538 | push | success | 2m23s | 89s | 372 MB restored |
| 40216270479 | dispatch, `config: demo` | success | 2m17s | 84s | 372 MB restored |

Demo-run steps: setup 1s, checkout 5s, demo copy <1s, Node 8s, Rust 12s, cache restore 18s, CLI 2s,
build 84s, list <1s, upload 1s, post steps 3s.

### Findings for the next template

1. **A `workflow_dispatch` choice plus a copy step is enough for multiple profiles.** No matrix, no
   separate workflow, no `tauri build --config` flag. `cp config.demo.json config.json` runs in Git
   Bash on `windows-latest`, and the build uses the normal embedded-config path. It worked first try.
2. **`github.event.inputs.config` is empty on `push`.** The `if: github.event.inputs.config == 'demo'`
   step is skipped cleanly. Do not use `inputs.config` in a workflow that also runs on push without
   checking the event; the event input map is the reliable one.
3. **The artifact name is fixed across profiles.** `FrontlineShell-Windows-Installer` is the same for
   production and demo. For this repo that is intentional, but a future template that needs both
   should name the artifact from the input. Always verify the profile by extracting the app and
   grepping for a config string, not by trusting the run title or artifact name.
4. **NSIS installers are directly inspectable with `7z`.** `7z x setup.exe` yields
   `frontline-desktop-shell.exe`; `strings -a` then shows the embedded `config.json`. This is the
   cheapest CI artifact proof on Linux and caught nothing only because the selector worked. The MSI
   is a WiX database and is not as easy to inspect.
5. **Pages needed no deploy workflow.** The legacy `main` / root source published `demo/` on the
   normal Pages build. Wait for the `pages build and deployment` run to finish before checking URLs;
   it took about 1m07s here.
6. **The earlier fixes held.** `tauri build --ci` with bundle targets in `tauri.conf.json` still
   avoids the PowerShell `msi,nsis` quoting failure, and the `v1` cache with no
   `cache-on-failure` still restores in ~18s. No new failure appeared in this pass.

### Not verified

- Installing or launching the demo-profile MSI/NSIS on Windows, including whether the two live Pages
  panes place correctly under WebView2. Linux still stacks them.
- The demo is static and does not simulate authentication, Power Apps runtime behavior, or tenant
  data. It only exercises the shell's URLs, allowlist host, and layout dimensions.

## 2026-10-25 — First real execution on Linux (playtest)

### Attempt

- No Xvfb or xdotool on this host, but a live Xorg display `:1` (1920x1080, KWin/Xwayland) was
  available, so the app ran there with `GDK_BACKEND=x11`, `WEBKIT_DISABLE_COMPOSITING_MODE=1`, and
  `LIBGL_ALWAYS_SOFTWARE=1`. A throwaway XTest helper sent the right-click and the shortcut; both it
  and the test pages were deleted afterwards.
- Built a debug binary with a temporary config: 1200x800 window, 400 px right sidebar,
  `allowedDomains: ["127.0.0.1"]`, and both panes pointed at local test pages on port 8766. The
  product `config.json` was restored byte-for-byte and matches the commit; the test binary was
  discarded.

### Observed

| Check | Result |
| --- | --- |
| Window creation | 1200x800 window titled `Frontline Playtest` appeared and stayed alive. |
| Page loads | Server logged `GET /primary.html` and `/secondary.html`; app logged both `[NAVIGATION][ALLOWED]`. |
| Allowlist | Auto-redirect to `http://example.com/blocked` logged `[SECURITY][BLOCKED] ... primary pane` and did not navigate. |
| Resize | `wmctrl` to 1000x600 logged `[LAYOUT][SYNC] Viewports snapped to: 1000x600`. |
| Context menu | Right-click produced `GET /ctx?pane=primary&prevented=true`; the injected suppression worked. |
| Supervisor exit | XTest `Ctrl+Shift+Q` logged `[SECURITY] Supervisor exit activated` and the process exited. |
| stderr | Empty throughout. |

### Real limitation found

Both pages reported the **same** viewport: 1142x380 at startup and 952x285 after the resize. They are
stacked vertically, each taking half the window, not placed at the computed primary/sidebar rectangles.
This is Tauri/Wry's Linux backend, not the layout maths: Tauri hands child webviews a `gtk::Box`, and
wry packs them with `pack_start(webview, true, true, 0)`; its `set_bounds` only positions a child when
the container is a `gtk::Fixed` (wry 0.57.1 `src/lib.rs:2031`, `src/webkitgtk/mod.rs:700-720`).

No code change was made. The product target is Windows/WebView2, where child bounds are honored, and
adding GTK-specific layout code would be platform sprawl for a Windows-only app. **Windows dual-pane
placement remains unverified.** Kiosk, SSO, installer launch, and memory use were not exercised.

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

### Deviations from the original brief's sample, and why

1. **`tauri-plugin-global-shortcut` added.** Tauri v2 has no core global-shortcut API, and FR-03
   requires `Ctrl+Shift+Q`. Registered Rust-side, so no frontend permission or capability is exposed.
   The official plugin requires Rust 1.90+, which is relevant to CI toolchain choice.
2. **`build.rs` parses the shared `config.rs` schema**, not just JSON syntax. A missing or misspelled
   required key is now a compile error too, which is stronger than the brief's runtime `expect`.
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

### What the original brief requires

The product is a **Tauri v2 Windows desktop shell**, not a web app. Requirements as understood:

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

- The original brief — 468 lines. Contained the product requirements, implementation guide,
  developer task list, and most of an `AGENTS.md`. **It was truncated mid-sentence at Part 4 rule 5**
  ("using c"); there was no missing tail on disk. Parts 1–3 were intact. It has since been absorbed
  into `agents.md` and removed from the repo.
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
  the sister-pattern equivalent of their wizard without adding a multi-step app the original brief
  never asked for.
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

- The original brief says "custom pixel or proportional split boundaries" but the schema only exposes pixel
  `sidebarWidth`. Implement pixel (schema is authoritative) and note the gap.
- The doc's `layout.mode` is a free string; treat anything other than `"dual"` as single, matching the
  sample code.
- Exact Windows installer behavior, SSO popups, kiosk, and RSS < 90 MB cannot be verified on this
  Linux host. These stay manual/handoff checks.
- Whether to point the GitHub repo's homepage at the Pages URL. Not required; decide at launch.

### Next

Phase 1 and 2 from `todo.md`: config, splash, Tauri scaffold, icons, `cargo check`. Then the workflow
and a first real CI run before the Pages page and README rewrite.
