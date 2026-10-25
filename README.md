# Powerapp Frontline Workspace Shell

A Tauri v2 Windows desktop shell for Power Apps, Fabric, and adjacent web tools. `config.json` is
compiled into the binary, so the endpoint gets one `.msi` or `.exe` with no loose configuration.
GitHub Actions builds both installers from a copied template.

[Template page](https://spuds0588.github.io/Powerapp-Frontline-Workspace-Shell/) &middot;
[Download the latest build](https://github.com/Spuds0588/Powerapp-Frontline-Workspace-Shell/actions/workflows/release.yml) &middot;
[Product spec](PRD-Powerapp%20Frontline%20Workplace%20Shell.md)

## Use this template

1. Click **Use this template** and create your repository.
2. Edit `config.json`: `title`, `panes`, `allowedDomains`, `layout`.
3. Commit. The push starts **Build Enterprise Desktop Shell**.
   A fresh copy may not run on its initial commit. If no run appears, open
   **Actions → Build Enterprise Desktop Shell → Run workflow**.
4. In the latest green run, download **FrontlineShell-Windows-Installer**, then run the `.msi` or
   `-setup.exe`.

Installers are build artifacts, not releases. Nothing is published to the Releases page.

## Configure

`config.json` is the only file to edit.

| Key | Purpose |
| --- | --- |
| `title`, `kioskMode`, `fullscreen`, `windowWidth`, `windowHeight` | Window identity and startup state. |
| `allowedDomains` | Apex domains and their subdomains allowed to load. Empty means allow all. |
| `layout.mode`, `sidebarPosition`, `sidebarWidth` | `dual` or `single`; `left` or `right`; sidebar pixels. |
| `panes.primary.url`, `panes.secondary.url` | The pages to mount. Secondary is optional. |

Invalid JSON or a missing required key fails the build: `src-tauri/build.rs` validates `config.json`
with the same schema the app uses at runtime. The full schema is in the
[product spec](PRD-Powerapp%20Frontline%20Workplace%20Shell.md).

## Where the installers land

- In the run: `src-tauri/target/release/bundle/msi/*.msi` and `src-tauri/target/release/bundle/nsis/*.exe`.
- Downloaded: `FrontlineShell_<version>_x64_en-US.msi` or `FrontlineShell_<version>_x64-setup.exe`.
- A cold copy builds in about six minutes; a warm cache in about two and a half.

## Verify and maintain

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
```

Build rules, the template-copy flow, and cleanup: [agents.md](agents.md).
CI timings, cache findings, and Windows limitations: [history.md](history.md).
Remaining work: [todo.md](todo.md). The supervisor exit is `Ctrl+Shift+Q`, including kiosk mode.

## License

MIT
