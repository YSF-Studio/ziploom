import { getPrefs, savePrefs } from "./prefs.js";

const MEDIA = window.matchMedia("(prefers-color-scheme: dark)");

function resolvedTheme(mode) {
  if (mode === "system") return MEDIA.matches ? "dark" : "light";
  return mode === "dark" ? "dark" : "light";
}

function applyResolved(resolved) {
  const root = document.documentElement;
  root.classList.remove("theme-light", "theme-dark");
  root.classList.add(resolved === "dark" ? "theme-dark" : "theme-light");
}

export function initTheme(mode = getPrefs().theme) {
  applyResolved(resolvedTheme(mode));
  const onChange = () => {
    if (getPrefs().theme === "system") applyResolved(resolvedTheme("system"));
  };
  MEDIA.addEventListener("change", onChange);
  return () => MEDIA.removeEventListener("change", onChange);
}

export function setTheme(mode) {
  savePrefs({ theme: mode });
  applyResolved(resolvedTheme(mode));
}

export function getTheme() {
  return getPrefs().theme;
}
