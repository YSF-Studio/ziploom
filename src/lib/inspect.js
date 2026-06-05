const EXEC_EXT = new Set(["exe", "dll", "bat", "cmd", "ps1", "sh", "msi", "scr", "com"]);
const SCRIPT_EXT = new Set(["js", "vbs", "wsf", "jar", "py", "rb", "php"]);

export function analyzeEntry(entry) {
  const flags = [];
  const name = entry.path.split("/").pop() ?? entry.path;
  const lower = name.toLowerCase();

  if (/\.[a-z0-9]{2,5}\.[a-z0-9]{2,5}$/i.test(name)) {
    flags.push("double-ext");
  }
  const ext = lower.includes(".") ? lower.split(".").pop() : "";
  if (EXEC_EXT.has(ext)) flags.push("executable");
  if (SCRIPT_EXT.has(ext)) flags.push("script");
  if (/startup|autorun|launch/i.test(entry.path)) flags.push("suspicious-path");
  if (name.startsWith(".")) flags.push("hidden");

  return flags;
}

export function isFlagged(entry, threatByPath = new Map()) {
  if (entry._folder || entry.isDir) return false;
  if (threatByPath.get(entry.path)) return true;
  const flags = analyzeEntry(entry);
  return flags.length > 0 || entry.magicMatch === false;
}

export function threatLevel(entry, threatByPath = new Map(), scanDone = false) {
  if (entry._folder || entry.isDir) return "none";
  const backend = threatByPath.get(entry.path);
  if (backend) {
    const sev = (backend.severity ?? "").toLowerCase();
    if (sev === "high") return "high";
    if (sev === "medium") return "medium";
    if (sev === "low") return "low";
  }
  const flags = analyzeEntry(entry);
  if (!flags.length && entry.magicMatch !== false) return scanDone ? "clear" : "none";
  if (flags.includes("executable") || flags.includes("double-ext") || entry.magicMatch === false) {
    return "high";
  }
  if (flags.includes("script") || flags.includes("suspicious-path")) return "medium";
  return "low";
}

export function threatLabel(level) {
  switch (level) {
    case "high": return "High";
    case "medium": return "Medium";
    case "low": return "Low";
    case "clear": return "Clear";
    default: return "—";
  }
}

export function riskSummary(entries) {
  let score = 0;
  for (const e of entries) {
    const flags = analyzeEntry(e);
    if (flags.includes("double-ext")) score += 3;
    if (flags.includes("executable")) score += 2;
    if (flags.includes("script")) score += 2;
    if (flags.includes("suspicious-path")) score += 2;
    if (flags.includes("hidden")) score += 1;
  }
  if (score >= 8) return "HIGH";
  if (score >= 3) return "MEDIUM";
  return "LOW";
}

export function filterEntries(entries, query) {
  const q = query.trim().toLowerCase();
  if (!q) return entries;
  return entries.filter((e) => e.path.toLowerCase().includes(q));
}

export function filterEntriesAdvanced(entries, { query = "", flaggedOnly = false, threatByPath = new Map() } = {}) {
  let list = filterEntries(entries, query);
  if (flaggedOnly) {
    list = list.filter((e) => isFlagged(e, threatByPath));
  }
  return list;
}

export function sortEntries(entries, key, dir) {
  const mul = dir === "desc" ? -1 : 1;
  return [...entries].sort((a, b) => {
    const av = a[key] ?? "";
    const bv = b[key] ?? "";
    if (typeof av === "number" && typeof bv === "number") return (av - bv) * mul;
    return String(av).localeCompare(String(bv)) * mul;
  });
}

export function buildTreeStructure(entries) {
  const root = { name: "", path: "", children: [], files: [] };
  const nodeMap = new Map([["", root]]);

  for (const e of entries.filter((x) => !x.isDir)) {
    const parts = e.path.replace(/\\/g, "/").split("/");
    let parentPath = "";
    for (let i = 0; i < parts.length - 1; i++) {
      const part = parts[i];
      const path = parentPath ? `${parentPath}/${part}` : part;
      if (!nodeMap.has(path)) {
        const node = { name: part, path, children: [], files: [] };
        nodeMap.set(path, node);
        nodeMap.get(parentPath).children.push(node);
      }
      parentPath = path;
    }
    nodeMap.get(parentPath).files.push(e);
  }

  const sortNode = (node) => {
    node.children.sort((a, b) => a.name.localeCompare(b.name));
    node.files.sort((a, b) => a.path.localeCompare(b.path));
    for (const c of node.children) sortNode(c);
  };
  sortNode(root);
  return root;
}

export function flattenTree(node, collapsed, depth = 0, out = []) {
  for (const child of node.children) {
    out.push({
      path: `${child.path}/`,
      isDir: true,
      _folder: true,
      depth,
      size: 0,
      modified: null,
      compressedSize: null,
    });
    if (!collapsed.has(child.path)) {
      flattenTree(child, collapsed, depth + 1, out);
    }
  }
  for (const f of node.files) {
    out.push({ ...f, depth, _file: true });
  }
  return out;
}

export function entropyPercent(entropy) {
  if (entropy == null || entropy === "") return 0;
  return Math.min(100, Math.max(0, (Number(entropy) / 8) * 100));
}

export async function copyText(text) {
  if (!text) return false;
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

export function exportReport(info, format) {
  const rows = info.entries ?? [];
  const header = {
    format: info.format,
    totalFiles: info.totalFiles,
    totalSize: info.totalSize,
    totalCompressed: info.totalCompressed,
    risk: riskSummary(rows),
  };
  if (format === "json") {
    return JSON.stringify({ ...header, entries: rows }, null, 2);
  }
  if (format === "csv") {
    const lines = [
      "path,type,size,compressed,modified,entropy,magic,detected_type,md5,sha1,sha256,flags",
    ];
    for (const e of rows) {
      const flags = analyzeEntry(e).join("|");
      const magic =
        e.magicMatch === true ? "match" : e.magicMatch === false ? "mismatch" : "";
      lines.push(
        `"${e.path}",${e.isDir ? "dir" : "file"},${e.size},${e.compressedSize ?? ""},"${e.modified ?? ""}",${e.entropy ?? ""},"${magic}","${e.detectedType ?? ""}","${e.md5 ?? ""}","${e.sha1 ?? ""}","${e.sha256 ?? ""}","${flags}"`
      );
    }
    return lines.join("\n");
  }
  return [
    `ZipLoom Inspect Report`,
    `Format: ${header.format}`,
    `Files: ${header.totalFiles}`,
    `Size: ${header.totalSize} bytes`,
    `Risk: ${header.risk}`,
    "",
    ...rows.map((e) => `- ${e.path} (${e.size} bytes)`),
  ].join("\n");
}

export function downloadText(filename, text) {
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
