export function formatSize(bytes) {
  if (bytes == null || Number.isNaN(bytes)) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function compRatio(entry) {
  if (!entry?.compressedSize || !entry?.size) return null;
  return ((1 - entry.compressedSize / entry.size) * 100).toFixed(0);
}

export function globalRatio(totalSize, totalCompressed) {
  if (!totalSize || !totalCompressed) return null;
  return ((1 - totalCompressed / totalSize) * 100).toFixed(1);
}

export function extForFormat(format) {
  const map = {
    zip: "zip",
    "7z": "7z",
    tar: "tar",
    "tar.gz": "tar.gz",
    "tar.bz2": "tar.bz2",
    "tar.xz": "tar.xz",
  };
  return map[format] ?? format;
}
