<script>
  import { basename } from "../path.js";
  import { formatSize } from "../format.js";

  let {
    archive = "",
    format = "",
    totalFiles = 0,
    totalSize = 0,
    risk = "",
    scanDone = false,
    busy = false,
    onBrowse,
    onClear,
  } = $props();
</script>

<div class="archive-bar">
  <div class="archive-bar-main">
    <span class="archive-icon" aria-hidden="true">📦</span>
    <div class="archive-meta">
      <span class="archive-name" title={archive}>{basename(archive)}</span>
      <span class="archive-sub">
        {#if format}<span class="fmt">{format.toUpperCase()}</span>{/if}
        {totalFiles} files · {formatSize(totalSize)}
        {#if risk}· Risk {risk}{/if}
        {#if scanDone}<span class="scan-badge">scanned</span>{/if}
      </span>
    </div>
  </div>
  <div class="archive-bar-actions">
    <button class="btn-link" disabled={busy} onclick={onBrowse}>Change</button>
    <button class="btn-link danger" disabled={busy} onclick={onClear}>Clear</button>
  </div>
</div>

<style>
  .archive-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    margin-bottom: 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    box-shadow: var(--shadow);
    flex-shrink: 0;
  }
  .archive-bar-main {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    flex: 1;
  }
  .archive-icon { font-size: 18px; flex-shrink: 0; }
  .archive-meta { min-width: 0; }
  .archive-name {
    display: block;
    font-size: 14px;
    font-weight: 700;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .archive-sub {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    font-size: 11px;
    color: var(--muted);
    margin-top: 2px;
  }
  .fmt {
    padding: 1px 6px;
    border-radius: 8px;
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }
  .scan-badge {
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }
  .archive-bar-actions { display: flex; gap: 8px; flex-shrink: 0; }
  .btn-link {
    border: none;
    background: none;
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    padding: 4px 6px;
  }
  .btn-link:hover { text-decoration: underline; }
  .btn-link:disabled { opacity: 0.45; cursor: not-allowed; }
  .btn-link.danger { color: var(--err); }
</style>
