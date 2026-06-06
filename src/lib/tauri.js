import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";
import { open as tauriOpen, save as tauriSave } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";

export { isTauri };

export async function invoke(cmd, args = {}) {
  if (!isTauri()) {
    throw new Error("This feature is only available in the ZipLoom app. Run: npm run tauri:dev");
  }
  try {
    return await tauriInvoke(cmd, args);
  } catch (err) {
    console.error(`[ZipLoom IPC] ${cmd} failed:`, err);
    throw err;
  }
}

export async function open(options) {
  if (!isTauri()) {
    throw new Error("File dialogs are only available in the ZipLoom app. Run: npm run tauri:dev");
  }
  return tauriOpen(options);
}

export async function save(options) {
  if (!isTauri()) {
    throw new Error("Save dialogs are only available in the ZipLoom app. Run: npm run tauri:dev");
  }
  return tauriSave(options);
}

export async function writeText(path, contents) {
  if (!isTauri()) {
    throw new Error("Writing files is only available in the ZipLoom app. Run: npm run tauri:dev");
  }
  return writeTextFile(path, contents);
}

/** Prefer webview window — required for close/minimize/drag in Tauri v2. */
export function getWindow() {
  if (!isTauri()) return null;
  try {
    return getCurrentWebviewWindow();
  } catch {
    return getCurrentWindow();
  }
}

export async function windowAction(action) {
  const win = getWindow();
  if (!win) return;
  if (action === "close") await win.close();
  else if (action === "minimize") await win.minimize();
  else if (action === "maximize") await win.toggleMaximize();
}

export function setupDragDrop(handler) {
  if (!isTauri()) return () => {};
  let disposed = false;
  let unlisten;
  getCurrentWebviewWindow()
    .onDragDropEvent(async (event) => {
      if (disposed || event.payload.type !== "drop") return;
      await handler(event.payload.paths ?? []);
    })
    .then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
  return () => {
    disposed = true;
    unlisten?.();
  };
}
