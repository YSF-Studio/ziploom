<script>
  import { formatSize } from "../format.js";
  import { analyzeEntry, threatLevel, threatLabel, entropyPercent, copyText } from "../inspect.js";

  let {
    rows = [],
    hashAlgo = "sha256",
    showEntropy = true,
    showMagic = true,
    showTimestamp = true,
    showHash = true,
    scanDone = false,
    selected = new Set(),
    focusedPath = "",
    threatByPath = new Map(),
    onToggleSelect,
    onRowClick,
    onToggleFolder,
    collapsed = new Set(),
  } = $props();

  function shortHash(h) {
    if (!h) return "—";
    return h.length > 12 ? `${h.slice(0, 12)}…` : h;
  }

  function hashFor(entry) {
    if (hashAlgo === "md5") return entry.md5;
    if (hashAlgo === "sha1") return entry.sha1;
    return entry.sha256;
  }

  function formatEntropy(v) {
    if (v == null || v === "") return "—";
    return Number(v).toFixed(2);
  }

  function formatMagic(entry) {
    if (entry._folder || entry.isDir) return "—";
    if (entry.detectedType) {
      const tag = entry.magicMatch === false ? "!" : entry.magicMatch === true ? "✓" : "";
      return `${entry.detectedType}${tag}`;
    }
    return entry.magicMatch === false ? "mismatch" : "—";
  }

  function formatModified(value) {
    if (!value) return "—";
    const s = String(value);
    if (/^\d+$/.test(s)) {
      const d = new Date(Number(s) * 1000);
      if (!Number.isNaN(d.getTime())) {
        return d.toLocaleDateString(undefined, { year: "2-digit", month: "short", day: "numeric" });
      }
    }
    if (s.length > 16) return s.slice(0, 16);
    return s;
  }

  function indent(depth = 0) {
    return `${Math.max(0, depth) * 14}px`;
  }

  function folderKey(entry) {
    return entry.path.replace(/\/$/, "");
  }

  function displayName(entry) {
    const parts = entry.path.replace(/\\/g, "/").split("/").filter(Boolean);
    if (entry._folder) return parts[parts.length - 1] ? `${parts[parts.length - 1]}/` : entry.path;
    return parts[parts.length - 1] || entry.path;
  }
</script>

<div class="table-wrap">
  <table class="data inspect-table">
    <thead>
      <tr>
        <th class="col-check sticky-col"></th>
        <th class="col-name sticky-col2">Name</th>
        <th class="col-size">Size</th>
        <th class="col-threat">Threat</th>
        {#if showHash}<th class="col-hash">{hashAlgo.toUpperCase()}</th>{/if}
        {#if showEntropy}<th class="col-ent">Entropy</th>{/if}
        {#if showMagic}<th class="col-magic">Magic</th>{/if}
        {#if showTimestamp}<th class="col-mod">Modified</th>{/if}
      </tr>
    </thead>
    <tbody>
      {#each rows as e (e.path)}
        {@const level = threatLevel(e, threatByPath, scanDone)}
        {@const flags = analyzeEntry(e)}
        <tr
          data-path={e.path}
          class:folder={e._folder || e.isDir}
          class:flagged={flags.length > 0 || e.magicMatch === false}
          class:selected={focusedPath === e.path}
          class:threat-high={level === "high"}
          class:threat-medium={level === "medium"}
          onclick={() => onRowClick?.(e)}
        >
          <td class="col-check sticky-col">
            {#if e._folder}
              <button
                class="fold-btn"
                aria-label={collapsed.has(folderKey(e)) ? "Expand folder" : "Collapse folder"}
                onclick={(ev) => { ev.stopPropagation(); onToggleFolder?.(folderKey(e)); }}
              >{collapsed.has(folderKey(e)) ? "▸" : "▾"}</button>
            {:else if !e.isDir}
              <input
                type="checkbox"
                checked={selected.has(e.path)}
                onclick={(ev) => ev.stopPropagation()}
                onchange={() => onToggleSelect?.(e.path)}
              />
            {/if}
          </td>
          <td class="col-name sticky-col2 name" title={e.path}>
            <span class="name-inner" style="padding-left: {indent(e.depth)}">{displayName(e)}</span>
          </td>
          <td class="col-size">{e.size ? formatSize(e.size) : "—"}</td>
          <td class="col-threat">
            <span class="threat-pill level-{level}">{e._folder || e.isDir ? "—" : threatLabel(level)}</span>
          </td>
          {#if showHash}
            <td class="col-hash mono" title={hashFor(e) ?? ""}>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <span
                class="hash-click"
                role="button"
                tabindex="0"
                onclick={(ev) => { ev.stopPropagation(); copyText(hashFor(e) ?? ""); }}
              >{shortHash(hashFor(e))}</span>
            </td>
          {/if}
          {#if showEntropy}
            <td class="col-ent">
              {#if e.entropy != null && !e._folder}
                <div class="ent-wrap" title={formatEntropy(e.entropy)}>
                  <div class="ent-bar" style="width: {entropyPercent(e.entropy)}%"></div>
                  <span>{formatEntropy(e.entropy)}</span>
                </div>
              {:else}—{/if}
            </td>
          {/if}
          {#if showMagic}
            <td class="col-magic" title={e.detectedType ?? ""}>{formatMagic(e)}</td>
          {/if}
          {#if showTimestamp}
            <td class="col-mod" title={e.modified ?? ""}>{formatModified(e.modified)}</td>
          {/if}
        </tr>
      {:else}
        <tr>
          <td colspan="12" class="empty-row">No entries match the current filter.</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .table-wrap {
    flex: 1;
    min-height: 0;
    overflow: auto;
    -webkit-overflow-scrolling: touch;
  }
  .inspect-table {
    width: 100%;
    min-width: max-content;
    border-collapse: collapse;
    table-layout: auto;
  }
  .inspect-table thead th {
    position: sticky;
    top: 0;
    z-index: 3;
    background: var(--surface-soft);
    padding: 8px 10px;
    text-align: left;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    color: var(--muted);
    font-weight: 600;
  }
  .inspect-table tbody td {
    padding: 7px 10px;
    text-align: left;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    background: var(--surface);
  }
  .sticky-col {
    position: sticky;
    left: 0;
    z-index: 2;
    min-width: 28px;
  }
  .sticky-col2 {
    position: sticky;
    left: 28px;
    z-index: 2;
    min-width: 180px;
  }
  .inspect-table thead .sticky-col,
  .inspect-table thead .sticky-col2 {
    z-index: 4;
    background: var(--surface-soft);
  }
  tr.selected td { background: var(--accent-soft) !important; }
  tr.threat-high td { color: var(--err); }
  tr.threat-medium td { color: var(--warn); }
  tr.flagged td.name { font-weight: 600; }
  .name-inner {
    display: inline-block;
    max-width: 360px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .fold-btn {
    border: none;
    background: none;
    cursor: pointer;
    color: var(--muted);
    font-size: 12px;
    padding: 0;
    width: 20px;
  }
  .threat-pill {
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 8px;
    background: var(--surface-soft);
  }
  .threat-pill.level-high { background: var(--err-soft); color: var(--err); }
  .threat-pill.level-medium { background: var(--warn-bg); color: var(--warn); }
  .threat-pill.level-low { background: var(--ok-soft); color: var(--ok); }
  .threat-pill.level-clear { background: var(--ok-soft); color: var(--ok); }
  .ent-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 72px;
  }
  .ent-bar {
    height: 4px;
    border-radius: 2px;
    max-width: 40px;
    background: linear-gradient(90deg, var(--teal), var(--accent));
  }
  .hash-click { cursor: pointer; }
  .hash-click:hover { color: var(--accent); text-decoration: underline; }
  .mono { font-family: var(--mono); font-size: 10px; }
  .empty-row {
    text-align: center;
    color: var(--muted);
    padding: 24px !important;
  }
</style>
