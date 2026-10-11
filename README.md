# Peachy Island — Coucou-based Windows app build pack

This is a **one-upload GitHub Actions build pack**, not a replacement toy prototype. When the workflow runs, GitHub downloads the current open-source Coucou repository, uses its actual `windows/` Tauri + Rust + TypeScript application, applies Peachy Island branding, and builds Windows `.exe` and `.msi` installers on GitHub's hosted Windows runner.

## One-time setup on your school laptop

1. Download this ZIP and extract it.
2. Open your GitHub repository.
3. Upload the **contents inside the extracted folder** (especially `.github/` and `branding/`) to the repository root. Do not upload the ZIP as a file.
4. Replace `.github/workflows/main.yml` if asked.
5. In the repository's `.github/workflows/` folder, delete old workflows that build the earlier prototype (for example `build-windows.yml`) so only this new workflow builds the app.
6. Commit changes. Then open **Actions → Build Peachy Island for Windows**. If it did not start automatically, click **Run workflow**.
7. After a green checkmark, open that run and download **Peachy-Island-Windows-Installers** under Artifacts. It contains the `.exe` and `.msi` installer(s).

No Node, Rust or C++ tools need to be installed on your school laptop. GitHub Actions builds remotely. School policy may still block installing or running desktop apps; follow your school's rules.

## What is included

The workflow builds the actual upstream Windows app, preserving the Windows implementation of agent/session monitoring, Claude Code permission approvals, chat, supported AI/provider connections, file drop, settings, shortcuts, integrations, tray actions and mascot interactions. These features still require the relevant local agent or service to be installed/configured; API-backed features need your own API key. Some features in Coucou are macOS-only and cannot be promised on Windows.

The branding patch changes the product name to **Peachy Island**, applies a blue-purple translucent glass skin, uses an orange/white icon and adds an orange cap overlay to the mascot canvas. It does not replace the upstream app's feature logic. Because the overlay is an adaptation, the mascot may need visual fine-tuning after you test the build on your machine.

## Important

- This pack has not been compiled in this chat environment. The first GitHub Actions run is the actual build test; if it fails, share the failed step log.
- The upstream repository is fetched at build time, so GitHub Actions needs network access to GitHub.
- Upstream code is MIT licensed; the original Coucou/Mochi name and original bundled character/sounds/assets are separately reserved by the upstream author. This community build uses the Peachy Island name and custom icon and is not an official Coucou release. Review upstream `LICENSE` and `LICENSE-ASSETS.md` before redistribution.
- Source: https://github.com/Louis-CFM/coucou
