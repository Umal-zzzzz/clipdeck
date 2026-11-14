# ClipDeck

<p align="center">
  <img src="src/assets/clipdeck-icon.png" width="128" alt="ClipDeck icon">
</p>

<p align="center">
  A lightweight, local-first clipboard history for macOS.
</p>

ClipDeck keeps your recent copied text and images available from the menu bar. Open the history with a keyboard shortcut, search it, and paste an earlier item without switching to another clipboard app.

> ClipDeck is an early preview. The current public build targets Apple Silicon Macs running macOS 12 or later.

## Download

Download the latest DMG from [GitHub Releases](https://github.com/Umal-zzzzz/clipdeck/releases/latest).

The current release is built for Apple Silicon. Intel support will require an Intel build.

## Install

1. Download the `.dmg` file from the latest release.
2. Open it and drag **ClipDeck** to the **Applications** folder.
3. Launch ClipDeck from Applications.

ClipDeck runs in the menu bar rather than the Dock. If macOS warns that the developer cannot be verified, Control-click the app, choose **Open**, and confirm. You may also need to choose **Open Anyway** in **System Settings → Privacy & Security**.

GitHub builds may not be notarized yet, so this warning is expected for early releases.

## How to use

- Copy text or an image in any app. ClipDeck records it automatically.
- Press **Control + Option + V** to open or hide the history.
- Click an item, or select it with the arrow keys and press **Enter**, to paste it into the previous app.
- Search the history from the search field.
- Use the menu bar icon to pause recording, open settings, or quit ClipDeck.

ClipDeck supports text and PNG/TIFF images. A screenshot copied with **Control + Shift + Command + 4** is added to the image history; the usual **Shift + Command + 4** saves the screenshot as a file instead.

## Permissions

ClipDeck needs macOS permission to return a selected item to the previous app automatically:

1. Open **System Settings → Privacy & Security → Accessibility**.
2. Allow **ClipDeck**.
3. If macOS asks whether ClipDeck may control **System Events**, review and allow the Automation permission.
4. Restart ClipDeck if the permission does not take effect immediately.

If these permissions are not granted, ClipDeck still copies the selected text to the clipboard, but automatic pasting may not work. Some protected input fields and applications may also block simulated paste actions.

## Privacy and stored data

- ClipDeck does not require an account and does not sync clipboard data to a server.
- Clipboard history is stored locally on this Mac at:
  `~/Library/Application Support/jp.umal.clipdeck/`
- The history can include passwords, tokens, or other sensitive information. Pause recording before copying sensitive data, or delete the history afterward.
- Settings include automatic history deletion, automatic clearing of the system clipboard, and a button to clear all saved history.
- ClipDeck currently stores up to 100 recent entries. Text and image size limits are applied to keep local storage bounded.

To remove stored data after uninstalling, quit ClipDeck and delete the `jp.umal.clipdeck` folder from `~/Library/Application Support/`.

## Troubleshooting

### No new items appear

Make sure recording is enabled in ClipDeck. Copy a new text or PNG/TIFF image after launching the app; existing clipboard contents are not imported automatically.

### Automatic paste does not work

Check the Accessibility and Automation permissions above, then restart ClipDeck. The selected text should still be available in the clipboard even when automatic paste fails.

### The app does not open after downloading

Use Control-click → **Open** once, or choose **Open Anyway** in **System Settings → Privacy & Security**. This is common for unsigned preview builds downloaded from GitHub.

## For contributors

ClipDeck is built with Rust, Tauri 2, React, and TypeScript. Development requires macOS, Node.js, npm, Rust, and Xcode Command Line Tools.

```bash
npm ci
npm run tauri dev
```

Before submitting changes, run the available checks:

```bash
npm run typecheck
npm run build
npm run check:rust
npm run format:rust:check
```

To create a local DMG:

```bash
npm run tauri build -- --bundles dmg
```

The DMG is created under `src-tauri/target/release/bundle/dmg/`.

## License

ClipDeck is licensed under the [MIT License](LICENSE).
