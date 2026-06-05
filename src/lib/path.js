export function basename(path) {
  const p = path.replace(/\\/g, "/");
  return p.split("/").pop() || p;
}

export function archiveBadge(path, format) {
  if (format) return `${format.toUpperCase()} archive`;
  const ext = basename(path).split(".").pop()?.toLowerCase() ?? "";
  if (ext === "gz" && path.endsWith(".tar.gz")) return "TAR.GZ archive";
  return ext ? `${ext.toUpperCase()} archive` : "Archive";
}
