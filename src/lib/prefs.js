const KEY = "ziploom.prefs.v2";

const DEFAULTS = {
  theme: "light",
  defaultFormat: "zip",
  defaultLevel: 6,
  defaultOutputFolder: "",
  confirmDestructive: true,
};

export function getPrefs() {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    return { ...DEFAULTS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULTS };
  }
}

export function savePrefs(patch) {
  const next = { ...getPrefs(), ...patch };
  localStorage.setItem(KEY, JSON.stringify(next));
  return next;
}
