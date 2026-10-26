# Powerapp Frontline Workspace Shell

A Tauri v2 Windows desktop shell for Power Apps, Teams, and adjacent web tools. `config.json` is
compiled into the binary, so the endpoint gets one `.msi` or `.exe` with no loose configuration.
GitHub Actions builds both installers from a copied template.

[Template page](https://spuds0588.github.io/Powerapp-Frontline-Workspace-Shell/) &middot;
[Download the demo installer](https://github.com/Spuds0588/Powerapp-Frontline-Workspace-Shell/releases/tag/demo-latest) &middot;
[Latest build](https://github.com/Spuds0588/Powerapp-Frontline-Workspace-Shell/actions/workflows/release.yml)

Windows 10/11 only. Linux and macOS builds are not shipped; the requirements and product blockers
are recorded in [history.md](history.md).

## Use this template

1. Click **Use this template** and create your repository.
2. Edit `config.json`: `title`, `panes`, `allowedDomains`, `layout`.
3. Commit. The push starts **Build Enterprise Desktop Shell**.
   A fresh copy may not run on its initial commit. If no run appears, open
   **Actions → Build Enterprise Desktop Shell → Run workflow**.
4. In the latest green run, download **FrontlineShell-Windows-Installer**, then run the `.msi` or
   `-setup.exe`.

Your installers are build artifacts, not releases. The only release in this repository is the
prebuilt demo. The artifact is tied to your repository and expires; download it when you need it.

## Agent handoff

The [template page](https://spuds0588.github.io/Powerapp-Frontline-Workspace-Shell/#agent) has a
copyable prompt for a coding agent. It tells the agent to read `agents.md`, ask for your app URLs,
domains, and layout, edit `config.json`, and walk you through the commit that starts the build. The
prompt is also in [index.html](index.html#agent).

## Demo build

The root page shows three reference pairings in a simulated desktop: Power Apps + Teams, Power Apps
+ Teams Phone, and Dynamics 365 + a Power Apps canvas app. Those are labeled Microsoft Learn
screenshots, not live tenant data. `demo/` holds the two static pages the installer actually loads.

- Download the unsigned demo installer from
  [Releases → demo-latest](https://github.com/Spuds0588/Powerapp-Frontline-Workspace-Shell/releases/tag/demo-latest).
  Windows SmartScreen may warn; the build is not code-signed.
- The installer opens the dispatch pairing: a Power Apps dashboard and a Teams recap. The demo NSIS
  installer was smoke-tested on Windows 10 LTSC: both panes loaded, the 420 px right sidebar matched
  the config, the context menu was suppressed, and `Ctrl+Shift+Q` closed the app.
- To rebuild it, run **Actions → Build Enterprise Desktop Shell → Run workflow** with
  **Configuration to embed** set to `demo`.

The demo is a test profile, not the shipped default. `config.json` still points at Power Apps and
Teams; CI copies `config.demo.json` over it only for a manual `config: demo` run.

## Configure

`config.json` is the only file to edit.

| Key | Purpose |
| --- | --- |
| `title`, `kioskMode`, `fullscreen`, `windowWidth`, `windowHeight` | Window identity and startup state. |
| `allowedDomains` | Apex domains and their subdomains allowed to load. Empty means allow all. |
| `layout.mode`, `sidebarPosition`, `sidebarWidth` | `dual` or `single`; `left` or `right`; sidebar pixels. |
| `panes.primary.url`, `panes.secondary.url` | The pages to mount. Secondary is optional. |

Invalid JSON or a missing required key fails the build: `src-tauri/build.rs` validates `config.json`
with the same schema the app uses at runtime. The schema is
[`src-tauri/src/config.rs`](src-tauri/src/config.rs).

## Where the installers land

- In the run: `src-tauri/target/release/bundle/msi/*.msi` and `src-tauri/target/release/bundle/nsis/*.exe`.
- Downloaded: `FrontlineShell_<version>_x64_en-US.msi` or `FrontlineShell_<version>_x64-setup.exe`.
- A cold copy builds in about six minutes; a warm cache in about two and a half.
- The demo release carries the same two files, built from `config.demo.json`.

## Verify and maintain

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
```

Build rules, the template-copy flow, and cleanup: [agents.md](agents.md).
CI timings, cache findings, Linux limitations, and the Linux/macOS research:
[history.md](history.md). Remaining work: [todo.md](todo.md). The supervisor exit is `Ctrl+Shift+Q`,
including kiosk mode.

## License

MIT
