<script>
  import DropzoneArea from "./DropzoneArea.svelte";
  import { invoke, open, save } from "../tauri.js";
  import { getPrefs } from "../prefs.js";
  import { filterSources } from "../compress.js";
  import { basename } from "../path.js";
  import { notify } from "../toast.js";

  let { onToast, onBusy } = $props();
  const prefs = getPrefs();

  let sources = $state([]);
  let format = $state(prefs.defaultFormat || "zip");
  let busy = $state(false);
  let result = $state(null);
  let usePassword = $state(false);
  let password = $state("");
  let cleanMac = $state(true);
  let level = $state(prefs.defaultLevel ?? 6);

  const formats = [
    { value: "zip", label: "ZIP — Universal format", ok: true },
    { value: "tar", label: "TAR — No compression", ok: true },
    { value: "tar.gz", label: "GZip — GZip compressed TAR", ok: true },
    { value: "tar.bz2", label: "BZip2 — BZip2 compressed TAR", ok: true },
    { value: "tar.xz", label: "XZ — XZ compressed TAR", ok: true },
    { value: "tar.zst", label: "Zstandard — Modern fast compression", ok: true },
  ];

  const levelLabel = $derived(
    level <= 2 ? "Fast" : level <= 5 ? "Normal" : level <= 7 ? "Good" : "Best"
  );

  function saveFilters(fmt) {
    const extMap = {
      zip: ["zip"],
      tar: ["tar"],
      "tar.gz": ["tar.gz", "tgz"],
      "tar.bz2": ["tar.bz2", "tbz2"],
      "tar.xz": ["tar.xz", "txz"],
      "tar.zst": ["tar.zst", "tzst"],
    };
    const exts = extMap[fmt] ?? [fmt];
    const label = formats.find((f) => f.value === fmt)?.label?.split("—")[0]?.trim() ?? fmt;
    return [{ name: label, extensions: exts }];
  }

  export async function addPaths(paths) {
    if (!paths?.length) return;
    try {
      const stats = await invoke("stat_paths", { paths });
      sources = [...sources, ...stats];
    } catch {
      sources = [...sources, ...paths.map((p) => ({ path: p, isDir: false, size: 0 }))];
    }
  }

  async function browseFiles() {
    const sel = await open({ directory: false, multiple: true });
    if (sel) await addPaths(Array.isArray(sel) ? sel : [sel]);
  }

  async function browseFolder() {
    const sel = await open({ directory: true, multiple: false });
    if (sel) await addPaths([sel]);
  }

  function removeSource(i) {
    sources = sources.filter((_, j) => j !== i);
  }

  async function compress() {
    if (!sources.length) return notify(onToast, "Add files or folders first", "error");
    const fmt = formats.find((f) => f.value === format);
    if (fmt && !fmt.ok) return notify(onToast, `Format ${format} is not supported for compression`, "error");
    if (usePassword && !password) return notify(onToast, "Enter a password", "error");

    const paths = filterSources(
      sources.map((s) => s.path),
      ".DS_Store\n__MACOSX",
      { cleanMacMeta: cleanMac, includeHidden: false }
    );
    if (!paths.length) return notify(onToast, "All items were filtered out", "error");

    busy = true;
    onBusy?.(true);
    result = null;
    try {
      const output = await save({
        title: "Save archive as",
        defaultPath: `archive.${format}`,
        filters: saveFilters(format),
      });
      if (!output) return;

      if (usePassword && format !== "zip") {
        return notify(onToast, "Password protection is only supported for ZIP", "error");
      }

      const res = await invoke("compress_files", {
        sources: paths,
        output,
        format,
        password: usePassword && password ? password : null,
      });
      const outputPath = res.outputPath ?? res.output_path;
      if (!outputPath) throw new Error("Compress succeeded but output path is missing");

      result = {
        ...res,
        outputPath,
        filesProcessed: res.filesProcessed ?? res.files_processed,
        totalSize: res.totalSize ?? res.total_size,
      };
      notify(onToast, res.message, "success");
    } catch (e) {
      notify(onToast, String(e), "error");
    } finally {
      busy = false;
      onBusy?.(false);
    }
  }

  function onDrop(e) {
    e.preventDefault();
    const paths = [...(e.dataTransfer?.files ?? [])].map((f) => f.path).filter(Boolean);
    if (paths.length) addPaths(paths);
  }
</script>

<div class="page compress-page">
  <DropzoneArea
    variant="files"
    title="Drop files or folders here"
    hint="Drag & drop files and folders, or use the links below"
    browseLabel="Browse files"
    folderLabel="Browse folder"
    changeLabel="Click to add more"
    disabled={busy}
    onBrowse={browseFiles}
    onBrowseFolder={browseFolder}
    ondrop={onDrop}
  />

  {#if sources.length}
    <div class="source-bar">
      {#each sources as s, i}
        <div class="source-chip">
          <span class="path" title={s.path}>{basename(s.path)}</span>
          <button class="btn-remove" onclick={() => removeSource(i)} aria-label="Remove">&times;</button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="settings-panel">
    <label class="field">
      Format
      <select bind:value={format} disabled={busy}>
        {#each formats as f}
          <option value={f.value} disabled={!f.ok}>{f.label}</option>
        {/each}
      </select>
    </label>

    <div class="settings-row check">
      <input type="checkbox" id="pw" bind:checked={usePassword} disabled={busy} />
      <label for="pw">Password</label>
      {#if usePassword}
        <input type="password" bind:value={password} placeholder="Enter password" style="flex:1" disabled={busy} />
      {/if}
    </div>

    <div class="settings-row check">
      <input type="checkbox" id="clean" bind:checked={cleanMac} disabled={busy} />
      <label for="clean">Clean macOS metadata (__MACOSX, .DS_Store)</label>
    </div>

    {#if format === "zip" && usePassword}
      <div class="info-banner">AES-256 password ZIP — compatible with 7-Zip, WinRAR, and common archive tools</div>
    {/if}
    {#if format !== "zip" && usePassword}
      <div class="info-banner warn">Password protection is only available for ZIP</div>
    {/if}

    <div class="slider-row">
      <span>Compress</span>
      <input type="range" min="0" max="9" bind:value={level} disabled={busy} />
      <span class="slider-label">{levelLabel}</span>
    </div>
  </div>

  <button class="btn-cta" disabled={busy || !sources.length} onclick={compress}>
    {busy ? "Compressing…" : "Compress"}
  </button>

  {#if result}
    <div class="result" style="margin-top:14px">
      {result.filesProcessed} files compressed to {result.outputPath}
      {#if usePassword && format === "zip"}(password-protected ZIP){/if}
    </div>
  {/if}
</div>
