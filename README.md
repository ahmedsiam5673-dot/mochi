# Mochi Island — Desktop Companion

A custom liquid-glass Windows desktop companion using your orange-and-white mascot style. It has a floating island window, a full dashboard, a wardrobe, streaming AI chat, an activity log, local agent events, and a Claude Code hook bridge for permission decisions.

**This is an original project and is not affiliated with Coucou.** It does not include Coucou's protected mascot, name, sound pack, or other assets.

## Included in this starter

- Liquid-glass island interface with custom mascot, blue-purple accents, and a larger dashboard.
- Pages for Home, Agent Sessions, Permissions, AI Chat, Activity Log, Wardrobe, Integrations, Settings, and About.
- Appearance options: blue/purple/pink/teal palettes, blur intensity, motion preference, mascot moods, and accessories.
- Preferences stored locally in the webview's local storage; Chat API keys are session-memory-only and never included in exports.
- Streaming chat adapter scaffolding for Google Gemini, Anthropic, OpenAI, and local Ollama.
- Native Rust HTTP listener on `127.0.0.1:49321` for local agent events.
- A Claude Code PowerShell hook bridge. `PreToolUse` waits for a decision and returns a supported hook permission result; `PostToolUse`, `Notification`, `Stop`, and `SessionStart` are logged.
- Custom event schema at `POST http://127.0.0.1:49321/event`, and `GET /health` for status checks.
- Exportable activity/settings JSON and a demo mode whose events are explicitly labelled.

## Important feature boundaries

This is a real desktop-app source project, but it is **not a clone of every Coucou integration**. The Claude Code hook path is the implemented native integration. Codex, Cursor, Gemini CLI, Antigravity, OpenCode and service cards are adapter/setup guides or extension points—not automatic live monitors in this version. GitHub notifications can be fetched through a user-provided read-only token. Stripe, n8n, Vercel, Resend, Notion, Cal.com, music controls, global hotkeys, Windows startup registration, OS keychain storage, and native audio need further provider-specific/native work. Settings that mention these are marked as preferences or guides and are not represented as fully implemented controls.

The Chat API key fields are kept in JavaScript memory for the current session, not persisted, but native OS credential-manager integration is not included yet. Treat API keys as sensitive.

## Requirements (Windows 10/11)

1. Node.js 20 or newer.
2. Rust stable: install using [rustup](https://rustup.rs/).
3. Microsoft C++ Build Tools with the **Desktop development with C++** workload.
4. WebView2 Runtime (normally installed on current Windows 10/11 systems).

## Run in development

Open PowerShell in this folder and run:

```powershell
npm install
npm run tauri:dev
```

To preview just the frontend in a browser (no real agent bridge and no real native floating window):

```powershell
npm run dev
```

Then open the local Vite address printed in the terminal. Choose **Try a demo** to explore the layout.

## Build a Windows installer

```powershell
npm install
npm run tauri:build
```

Installer artifacts are produced under `src-tauri/target/release/bundle/` (NSIS `.exe` and/or MSI depending on your installed build tools). The current environment used to prepare this project did not have Rust/Cargo or Windows build tools available, so a compiled `.exe` is not included in this ZIP.

## Connect Claude Code

1. Start Mochi Island and keep the small island window open.
2. In this project folder, run:

   ```powershell
   powershell -ExecutionPolicy Bypass -File .\scripts\install-claude-hooks.ps1
   ```

3. The installer creates a timestamped backup of `~/.claude/settings.json` and preserves unrelated hook entries.
4. Start or continue a Claude Code session. Events are sent to the local app. `PreToolUse` requests can appear in the Permission Center and be allowed or denied from Mochi.
5. To remove only Mochi hook entries later, run:

   ```powershell
   powershell -ExecutionPolicy Bypass -File .\scripts\uninstall-claude-hooks.ps1
   ```

The hook script fails open if Mochi is not running, so a missing local app will not permanently block Claude Code.

## Custom agent events

Send sanitized JSON to the loopback bridge. Do not send secrets, API keys, full file contents, or sensitive prompts unless you intend to display them in the local activity log.

```json
{
  "hook_event_name": "AgentEvent",
  "session_id": "session-123",
  "tool_name": "Read",
  "tool_input": { "file_path": "src/App.tsx" },
  "summary": "Read src/App.tsx"
}
```

Use `POST http://127.0.0.1:49321/event` with `Content-Type: application/json`.

## AI chat

Go to **Settings → Chat providers**, select Gemini, Anthropic, OpenAI, or Ollama, then enter a key/base URL for the current session. Keys are cleared on application reload. A provider may block a request because of account permissions, CORS, API availability, or model access; those errors are shown in the chat window.

For Ollama, install and run Ollama locally, pull a model, and allow local requests from the WebView if your Ollama configuration requires an origin allowlist.

## Security notes

- The hook listener binds to `127.0.0.1`, never `0.0.0.0`.
- The installer backs up Claude settings and leaves unrelated existing hooks intact.
- The app has no telemetry implementation and does not upload files dropped on it.
- Inspect tool requests before allowing them. Do not automatically approve commands you do not understand.
- The hook receiver is intended for a trusted local machine. Do not expose its port publicly or through a remote tunnel without adding authentication.


## One-command Windows build helper

After the prerequisites above are installed, open PowerShell in the project folder and run:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-windows.ps1
```

The helper checks for Node.js and Rust, installs npm dependencies, and starts the Tauri installer build. If dependencies are already installed, use `-SkipInstall`. A build still requires network access the first time and a supported Windows build environment.

## Permission timeout behavior

If a live Claude Code `PreToolUse` request reaches Mochi but no one decides before the app's 120-second wait expires, Mochi returns **deny** rather than silently approving it. If the app is not running or its local hook endpoint cannot be reached, the PowerShell hook returns an empty response so Claude Code can use its usual permission flow instead of being stuck.

## Build on GitHub Actions (recommended for school-managed laptops)

This repository includes `.github/workflows/build-windows.yml`. GitHub's hosted Windows runner installs the build tools and compiles the Windows installers remotely, so you do not need Rust, Node.js, or C++ build tools installed on your own laptop.

### If you can upload the project files into a GitHub repository

1. Create a repository on GitHub. Keep it **Private** if you do not want the source publicly visible.
2. Upload the **contents** of this project folder to the repository root (so `package.json`, `src-tauri/`, and `.github/workflows/build-windows.yml` are at the top level), then commit the files to `main`.
3. Open the repository's **Actions** tab and select **Build Mochi Island for Windows**. If it did not start on its own, select **Run workflow**.
4. Wait for the run to finish successfully. Open that run, scroll to **Artifacts**, and download `Mochi-Island-Windows-Installers`.
5. Extract the downloaded artifact. The `nsis` folder should contain a Windows installer `.exe`; the `msi` folder may contain an `.msi` installer.

### Safety and limits

- Never put API keys, access tokens, or passwords in the repository or workflow file.
- The first build can take several minutes because dependencies are downloaded on the hosted runner.
- GitHub Actions usage limits depend on your GitHub plan and whether the repository is public or private.
- This workflow compiles the project; it does not code-sign the installer. Windows SmartScreen may show a warning for an unsigned app.
