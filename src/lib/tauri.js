import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";
import { open as tauriOpen, save as tauriSave } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";

export { isTauri };

export async function invoke(cmd, args = {}) {
  if (!isTauri()) {
    throw new Error("Fitur ini hanya tersedia di aplikasi ZipLoom. Jalankan: npm run tauri:dev");
  }
  return tauriInvoke(cmd, args);
}

export async function open(options) {
  if (!isTauri()) {
    throw new Error("Dialog file hanya tersedia di aplikasi ZipLoom. Jalankan: npm run tauri:dev");
  }
  return tauriOpen(options);
}

export async function save(options) {
  if (!isTauri()) {
    throw new Error("Dialog simpan hanya tersedia di aplikasi ZipLoom. Jalankan: npm run tauri:dev");
  }
  return tauriSave(options);
}

export async function writeText(path, contents) {
  if (!isTauri()) {
    throw new Error("Menulis file hanya tersedia di aplikasi ZipLoom. Jalankan: npm run tauri:dev");
  }
  return writeTextFile(path, contents);
}

export function getWindow() {
  if (!isTauri()) return null;
  return getCurrentWindow();
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
