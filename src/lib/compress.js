const SUPPORTED = new Set(["zip", "tar", "tar.gz"]);

export function isFormatSupported(format) {
  return SUPPORTED.has(format);
}

export function parseExcludePatterns(text) {
  return text
    .split(/[\n,]+/)
    .map((s) => s.trim())
    .filter(Boolean);
}

function matchPattern(path, pattern) {
  if (pattern.includes("*")) {
    const re = new RegExp("^" + pattern.replace(/\./g, "\\.").replace(/\*/g, ".*") + "$", "i");
    const base = path.split("/").pop() ?? path;
    return re.test(base) || re.test(path);
  }
  return path.includes(pattern);
}

export function filterSources(paths, patterns, { cleanMacMeta = true, includeHidden = false } = {}) {
  const excludes = [...parseExcludePatterns(patterns)];
  if (cleanMacMeta) {
    excludes.push(".DS_Store", "__MACOSX");
  }
  return paths.filter((p) => {
    const base = p.split("/").pop() ?? p;
    if (!includeHidden && base.startsWith(".")) return false;
    return !excludes.some((pat) => matchPattern(p, pat));
  });
}

export function estimateRatio(level) {
  const map = { 0: "5%", 3: "25%", 6: "40%", 9: "55%" };
  return map[level] ?? `${Math.min(55, 10 + level * 5)}%`;
}
