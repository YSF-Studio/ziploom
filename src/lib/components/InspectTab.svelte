<script>
  import DropzoneArea from "./DropzoneArea.svelte";
  import PasswordDialog from "./PasswordDialog.svelte";
  import ProgressBar from "./ProgressBar.svelte";
  import InspectArchiveBar from "./InspectArchiveBar.svelte";
  import InspectVirtualTable from "./InspectVirtualTable.svelte";
  import InspectDetailPanel from "./InspectDetailPanel.svelte";
  import { withProgress } from "../progressPoll.js";
  import { invoke, open, save, writeText } from "../tauri.js";
  import { basename, archiveBadge } from "../path.js";
  import {
    riskSummary,
    filterEntriesAdvanced,
    sortEntries,
    sortByTimestamp,
    exportReport,
    buildTreeStructure,
    flattenTree,
    isFlagged,
    isMagicMismatch,
    isSuspiciousTimestamp,
  } from "../inspect.js";
  import { notify } from "../toast.js";
  import { promptPassword, withArchivePassword } from "../password.js";

  let { onToast, onBusy } = $props();

  let archive = $state("");
  let busy = $state(false);
  let info = $state(null);
  let error = $state("");

  let viewMode = $state("tree");
  let sortKey = $state("path");
  let sortDir = $state("asc");
  let query = $state("");
  let flaggedOnly = $state(false);
  let mismatchOnly = $state(false);
  let suspiciousTimestampOnly = $state(false);
  let selected = $state(new Set());
  let collapsed = $state(new Set());

  let hashAlgo = $state("sha256");
  let showHash = $state(true);
  let showEntropy = $state(true);
  let showMagic = $state(true);
  let showTimestamp = $state(true);
  let showColMenu = $state(false);

  let archiveHashes = $state(null);
  let scanDone = $state(false);
  let forensicMeta = $state(null);
  let progress = $state(null);

  let focusedEntry = $state(null);
  let preview = $state(null);
  let previewLoading = $state(false);
  let findingsTab = $state("threats");

  let pwOpen = $state(false);
  let pwHandlers = $state(null);

  function askPassword() {
    return promptPassword(({ resolve, cancel }) => {
      pwHandlers = { resolve, cancel };
      pwOpen = true;
    });
  }

  function normalizeEntry(e) {
    return {
      path: e.path,
      size: e.size,
      compressedSize: e.compressedSize ?? e.compressed_size,
      isDir: e.isDir ?? e.is_dir,
      modified: e.modified,
      timestamp: e.timestamp ?? e.modified ?? null,
      timestampKind: e.timestampKind ?? e.timestamp_kind ?? null,
      md5: e.md5 ?? null,
      sha1: e.sha1 ?? e.sha_1 ?? null,
      sha256: e.sha256 ?? null,
      entropy: e.entropy ?? null,
      magicMatch: e.magicMatch ?? e.magic_match ?? null,
      detectedType: e.detectedType ?? e.detected_type ?? null,
      expectedType: e.expectedType ?? e.expected_type ?? null,
    };
  }

  function normalizeInfo(raw) {
    if (!raw) return null;
    const entries = (raw.entries ?? []).map(normalizeEntry);
    return {
      format: raw.format,
      entries,
      totalFiles: raw.totalFiles ?? raw.total_files ?? entries.length,
      totalSize: raw.totalSize ?? raw.total_size ?? 0,
      totalCompressed: raw.totalCompressed ?? raw.total_compressed,
    };
  }

  function mergeForensicReport(report) {
    if (!info || !report) return;
    const byPath = new Map((report.entries ?? []).map((e) => [e.path, normalizeEntry(e)]));
    info = {
      ...info,
      entries: info.entries.map((e) => {
        const scanned = byPath.get(e.path);
        return scanned ? { ...e, ...scanned } : e;
      }),
    };
    forensicMeta = {
      riskLabel: report.risk_label ?? report.riskLabel,
      riskScore: report.risk_score ?? report.riskScore,
      threats: report.threats ?? [],
      anomalies: report.anomalies ?? [],
    };
    scanDone = true;
  }

  const risk = $derived(
    forensicMeta?.riskLabel ?? (info ? riskSummary(info.entries ?? []) : null)
  );

  const threatByPath = $derived.by(() => {
    const map = new Map();
    for (const t of forensicMeta?.threats ?? []) map.set(t.file, t);
    return map;
  });

  const flaggedCount = $derived(
    info ? info.entries.filter((e) => isFlagged(e, threatByPath)).length : 0
  );

  const displayEntries = $derived.by(() => {
    if (!info) return [];
    let files = filterEntriesAdvanced(info.entries ?? [], {
      query,
      flaggedOnly,
      threatByPath,
    });
    if (mismatchOnly) files = files.filter((e) => isMagicMismatch(e));
    if (suspiciousTimestampOnly) files = files.filter((e) => isSuspiciousTimestamp(e.timestamp ?? e.modified));
    if (viewMode === "tree") {
      const tree = buildTreeStructure(files);
      return flattenTree(tree, collapsed);
    }
    if (sortKey === "timestamp") return sortByTimestamp(files, sortDir);
    return sortEntries(files, sortKey, sortDir);
  });

  const statusLine = $derived.by(() => {
    if (!info) return "";
    const parts = [`${info.totalFiles} files`];
    if (flaggedCount) parts.push(`${flaggedCount} flagged`);
    if (scanDone) parts.push("scanned");
    if (progress && !progress.is_done && busy) {
      parts.push(`${Math.round(progress.percent)}%`);
    }
    return parts.join(" · ");
  });

  async function loadArchive(password = null) {
    return normalizeInfo(await invoke("inspect_archive", { path: archive, password }));
  }

  export async function inspectPath(path) {
    if (!path) return;
    archive = path;
    archiveHashes = null;
    scanDone = false;
    forensicMeta = null;
    selected = new Set();
    focusedEntry = null;
    preview = null;
    collapsed = new Set();
    await inspect();
  }

  async function browse() {
    const sel = await open({ directory: false, multiple: false });
    if (sel) await inspectPath(sel);
  }

  async function inspect() {
    if (!archive) return notify(onToast, "Select an archive first", "error");
    busy = true;
    onBusy?.(true);
    info = null;
    error = "";
    forensicMeta = null;
    scanDone = false;
    preview = null;
    focusedEntry = null;
    try {
      info = await withArchivePassword(archive, invoke, askPassword, loadArchive);
      notify(onToast, `${info.totalFiles} entries loaded`, "success");
    } catch (e) {
      error = String(e);
      notify(onToast, error, "error");
    } finally {
      busy = false;
      onBusy?.(false);
    }
  }

  async function fullScan() {
    if (!archive) return browse();
    if (!info) await inspect();
    busy = true;
    onBusy?.(true);
    progress = { percent: 0, status: "Starting forensic scan…", is_done: false };
    try {
      const report = await withProgress(invoke, async () => {
        return await withArchivePassword(
          archive,
          invoke,
          askPassword,
          (pw) => invoke("forensic_scan_archive", { path: archive, password: pw })
        );
      }, (p) => { progress = p; });
      mergeForensicReport(report);
      const threats = report.threats?.length ?? 0;
      const anomalies = report.anomalies?.length ?? 0;
      const label = report.risk_label ?? report.riskLabel ?? "Unknown";
      notify(
        onToast,
        threats + anomalies > 0
          ? `Scan complete — ${label} (${threats} threats, ${anomalies} anomalies)`
          : `Scan complete — ${label}`,
        threats > 0 ? "error" : "success"
      );
    } catch (e) {
      scanDone = false;
      notify(onToast, String(e), "error");
    } finally {
      busy = false;
      onBusy?.(false);
    }
  }

  async function hashAll() {
    if (!archive) return browse();
    busy = true;
    onBusy?.(true);
    progress = { percent: 0, status: "Computing archive hashes…", is_done: false };
    try {
      archiveHashes = await withProgress(
        invoke,
        () => invoke("hash_archive", { path: archive }),
        (p) => { progress = p; }
      );
      notify(onToast, "Archive hashes computed", "success");
    } catch (e) {
      notify(onToast, String(e), "error");
    } finally {
      busy = false;
      onBusy?.(false);
    }
  }

  async function exportCsv() {
    if (!info) return notify(onToast, "Inspect an archive first", "error");
    try {
      const path = await save({
        defaultPath: "ziploom-inspect.csv",
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!path) return;
      await writeText(path, exportReport({
        ...info,
        risk: forensicMeta?.riskLabel ?? riskSummary(info.entries ?? []),
        scanDone,
      }, "csv"));
      notify(onToast, `CSV exported to ${path}`, "success");
    } catch (e) {
      notify(onToast, String(e), "error");
    }
  }

  async function extractSelected() {
    if (!archive) return notify(onToast, "Select an archive first", "error");
    const paths = [...selected];
    if (!paths.length) return notify(onToast, "Select files first (checkbox)", "error");
    const dir = await open({ directory: true, multiple: false, title: "Extraction destination folder" });
    if (!dir) return;
    busy = true;
    onBusy?.(true);
    progress = { percent: 0, status: "Extracting archive…", is_done: false };
    try {
      const res = await withProgress(invoke, async () => {
        return await withArchivePassword(
          archive,
          invoke,
          askPassword,
          (pw) =>
            invoke("extract_archive_entries", {
              archivePath: archive,
              outputDir: dir,
              paths,
              password: pw,
            })
        );
      }, (p) => { progress = p; });
      notify(onToast, `${res.message} → ${res.outputPath ?? res.output_path}`, "success");
    } catch (e) {
      notify(onToast, String(e), "error");
    } finally {
      busy = false;
      onBusy?.(false);
    }
  }

  async function loadPreview(entry) {
    if (!entry || entry._folder || entry.isDir) return;
    previewLoading = true;
    preview = null;
    try {
      preview = await withArchivePassword(
        archive,
        invoke,
        askPassword,
        (pw) =>
          invoke("preview_archive_entry", {
            archivePath: archive,
            entryPath: entry.path,
            password: pw,
          })
      );
    } catch (e) {
      notify(onToast, String(e), "error");
    } finally {
      previewLoading = false;
    }
  }

  function toggleSelect(path) {
    const next = new Set(selected);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    selected = next;
  }

  function toggleFolder(path) {
    const next = new Set(collapsed);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    collapsed = next;
  }

  function onRowClick(entry) {
    if (entry._folder) {
      toggleFolder(entry.path.replace(/\/$/, ""));
      return;
    }
    focusedEntry = entry;
    preview = null;
  }

  function jumpToPath(path) {
    const entry = info?.entries?.find((e) => e.path === path);
    if (entry) {
      focusedEntry = entry;
      preview = null;
      flaggedOnly = false;
      query = "";
      requestAnimationFrame(() => {
        const row = document.querySelector(`.inspect-table tr[data-path="${path}"]`);
        row?.scrollIntoView?.({ block: "nearest", behavior: "smooth" });
      });
    }
  }

  function clearArchive() {
    archive = "";
    info = null;
    error = "";
    archiveHashes = null;
    forensicMeta = null;
    scanDone = false;
    selected = new Set();
    focusedEntry = null;
    preview = null;
    collapsed = new Set();
  }

  function onDrop(e) {
    e.preventDefault();
    const p = e.dataTransfer?.files?.[0]?.path;
    if (p) inspectPath(p);
  }
</script>

<div class="page inspect-page">
  {#if !info}
    <DropzoneArea
      variant="search"
      title="Drop archive here for forensic inspection"
      hint="Load metadata first, then Full Scan for per-file hashes, entropy, and magic bytes"
      browseLabel="Choose archive"
      changeLabel="Click to change archive"
      fileName={archive ? basename(archive) : ""}
      badge={archive && info ? archiveBadge(archive, info.format) : ""}
      disabled={busy}
      onBrowse={browse}
      onClear={clearArchive}
      ondrop={onDrop}
    />
  {:else}
    <InspectArchiveBar
      {archive}
      format={info.format}
      totalFiles={info.totalFiles}
      totalSize={info.totalSize}
      {risk}
      {scanDone}
      {busy}
      onBrowse={browse}
      onClear={clearArchive}
    />
  {/if}

  <div class="action-bar">
    <button class="action-chip" disabled={busy || !archive} onclick={fullScan} title="Full forensic scan">
      🔬 Full Scan
    </button>
    <button class="action-chip" disabled={busy || !archive} onclick={hashAll} title="Hash archive container">
      # Hash Archive
    </button>
    <button class="action-chip" disabled={busy || !info} onclick={exportCsv} title="Export CSV report">
      ⬇ CSV
    </button>
    <span class="action-spacer"></span>
    {#if info}
      <span class="status-line">{statusLine}</span>
    {/if}
  </div>

  {#if error}<div class="result error">{error}</div>{/if}
  <ProgressBar {progress} />

  {#if info}
    <div class="inspect-body">
      <div class="inspect-toolbar">
        <div class="seg-control">
          <button class:active={viewMode === "tree"} onclick={() => (viewMode = "tree")}>Tree</button>
          <button class:active={viewMode === "flat"} onclick={() => (viewMode = "flat")}>Flat</button>
        </div>
        <input
          class="search-input"
          type="search"
          placeholder="Filter files…"
          bind:value={query}
          aria-label="Filter files"
        />
        <label class="filter-flag">
          <input type="checkbox" bind:checked={flaggedOnly} />
          Flagged only
        </label>
        <label class="filter-flag">
          <input type="checkbox" bind:checked={mismatchOnly} />
          Magic mismatch only
        </label>
        <label class="filter-flag">
          <input type="checkbox" bind:checked={suspiciousTimestampOnly} />
          Suspicious timestamp only
        </label>
        <label>
          Sort
          <select bind:value={sortKey}>
            <option value="path">Name</option>
            <option value="size">Size</option>
            <option value="entropy">Entropy</option>
            <option value="timestamp">Timestamp</option>
          </select>
        </label>
        <select bind:value={sortDir} aria-label="Sort direction">
          <option value="asc">Asc</option>
          <option value="desc">Desc</option>
        </select>
        {#if showHash}
          <select bind:value={hashAlgo} aria-label="Hash algorithm">
            <option value="md5">MD5</option>
            <option value="sha1">SHA1</option>
            <option value="sha256">SHA256</option>
          </select>
        {/if}
        <div class="col-menu-wrap">
          <button class="col-menu-btn" onclick={() => (showColMenu = !showColMenu)}>Columns ▾</button>
          {#if showColMenu}
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <div class="col-menu" role="menu" tabindex="0" onclick={(e) => e.stopPropagation()}>
              <label><input type="checkbox" bind:checked={showHash} /> Hash</label>
              <label><input type="checkbox" bind:checked={showEntropy} /> Entropy</label>
              <label><input type="checkbox" bind:checked={showMagic} /> Magic / mismatch</label>
              <label><input type="checkbox" bind:checked={showTimestamp} /> Timestamp</label>
            </div>
          {/if}
        </div>
      </div>

      <div class="inspect-split">
        <div class="inspect-main">
          <div class="table-panel">
            <InspectVirtualTable
              rows={displayEntries}
              {hashAlgo}
              {showHash}
              {showEntropy}
              {showMagic}
              {showTimestamp}
              {scanDone}
              {selected}
              focusedPath={focusedEntry?.path ?? ""}
              {threatByPath}
              {busy}
              {collapsed}
              onToggleSelect={toggleSelect}
              onRowClick={onRowClick}
              onToggleFolder={toggleFolder}
            />
          </div>
          <div class="extract-row">
            <button class="btn-secondary primary" disabled={busy || !selected.size} onclick={extractSelected}>
              Extract Selected ({selected.size})
            </button>
          </div>
        </div>

        <InspectDetailPanel
          entry={focusedEntry}
          {preview}
          {previewLoading}
          {archiveHashes}
          threats={forensicMeta?.threats ?? []}
          anomalies={forensicMeta?.anomalies ?? []}
          {scanDone}
          {threatByPath}
          {findingsTab}
          onFindingsTab={(t) => (findingsTab = t)}
          onJumpTo={jumpToPath}
          onPreview={loadPreview}
        />
      </div>
    </div>
  {:else if !archive}
    <div class="empty-hint">
      <p>Load an archive to inspect its contents.</p>
      <p class="sub">Full Scan adds per-file MD5/SHA1/SHA256, Shannon entropy, magic-byte mismatch detection, and timestamp awareness.</p>
    </div>
  {/if}
</div>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<svelte:window onclick={() => (showColMenu = false)} />

<PasswordDialog
  bind:open={pwOpen}
  title="Encrypted archive"
  message="This archive is password protected. Enter the password to inspect or extract."
  onConfirm={(pw) => pwHandlers?.resolve(pw)}
  onCancel={() => pwHandlers?.cancel()}
/>

<style>
  .action-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 10px;
    flex-shrink: 0;
  }
  .action-chip {
    padding: 7px 12px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    color: var(--text);
  }
  .action-chip:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
  .action-chip:disabled { opacity: 0.45; cursor: not-allowed; }
  .action-spacer { flex: 1; }
  .status-line { font-size: 11px; color: var(--muted); }
  .inspect-toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-bottom: 10px;
    flex-shrink: 0;
  }
  .search-input {
    min-width: 140px;
    flex: 1;
    max-width: 220px;
    padding: 6px 10px;
    font-size: 12px;
  }
  .filter-flag {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    cursor: pointer;
  }
  .col-menu-wrap { position: relative; }
  .col-menu-btn {
    padding: 6px 10px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
    font-size: 12px;
    cursor: pointer;
  }
  .col-menu {
    position: absolute;
    top: 100%;
    right: 0;
    z-index: 10;
    margin-top: 4px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    box-shadow: var(--shadow);
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    min-width: 140px;
  }
  .col-menu label { display: flex; align-items: center; gap: 6px; cursor: pointer; }
  .inspect-split {
    display: flex;
    flex: 1;
    min-height: 0;
    gap: 12px;
  }
  .inspect-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .table-panel {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .extract-row {
    margin-top: 10px;
    flex-shrink: 0;
  }
  .empty-hint {
    text-align: center;
    padding: 24px;
    color: var(--muted);
    font-size: 13px;
  }
  .empty-hint .sub { font-size: 12px; margin-top: 6px; }
</style>
