<script>
import { invoke } from "@tauri-apps/api/core";
import FilePreview from "./FilePreview.svelte";

let { activeCase, busy, msg, timeoutPromise, density, onFileSelect } = $props();
let imagePath = $state("");
let entries = $state([]);
let previewFile = $state(null);
let showPreview = $state(true);
let sortCol = $state(null);
let sortDir = $state("asc");
let previewRef = $state(null);

async function loadMft() {
  if (!imagePath) return;
  busy = true;
  try {
    entries = await timeoutPromise(invoke("parse_mft", { imagePath }), 60000);
    msg = `✅ ${entries.length} entries loaded`;
  } catch(e) { msg = `❌ ${typeof e === "string" ? e : String(e)}`; }
  busy = false;
}

function sortBy(col) {
  if (sortCol === col) { sortDir = sortDir === "asc" ? "desc" : "asc"; }
  else { sortCol = col; sortDir = "asc"; }
  entries = [...entries].sort((a, b) => {
    let va = a[col], vb = b[col];
    if (typeof va === "string") va = va.toLowerCase();
    if (typeof vb === "string") vb = vb.toLowerCase();
    if (va < vb) return sortDir === "asc" ? -1 : 1;
    if (va > vb) return sortDir === "asc" ? 1 : -1;
    return 0;
  });
}

function selectFile(entry) {
  const path = entry.filename ? imagePath.replace(/\/[^/]+$/, "/") + entry.filename : imagePath;
  previewFile = entry.filename || "unnamed";
  if (previewRef) previewRef.loadPreview(imagePath);
  if (onFileSelect) onFileSelect(imagePath);
}

function formatTime(t) {
  if (!t || t === "—") return "—";
  return t.substring(0, 19);
}

function sizeStr(s) {
  if (!s) return "—";
  const kb = s / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  const mb = kb / 1024;
  return `${mb.toFixed(1)} MB`;
}

const densityRows = { compact: "24px", standard: "32px", comfortable: "44px" };
const densityFont = { compact: "11px", standard: "12px", comfortable: "13px" };

function getRowStyle(i) {
  const h = densityRows[density] || densityRows.compact;
  const bg = i % 2 === 0 ? "transparent" : "rgba(255,255,255,0.02)";
  return `height:${h};font-size:${densityFont[density] || "11px"};background:${bg}`;
}

function sortIndicator(col) {
  if (sortCol !== col) return "";
  return sortDir === "asc" ? " ▲" : " ▼";
}
</script>

<div class="file-browser">
  <div class="toolbar">
    <h3>🗂️ File Browser (NTFS)</h3>
    <div class="toolbar-right">
      <button class="preview-toggle" class:active={showPreview} onclick={() => showPreview = !showPreview}>
        {showPreview ? "🔍 Preview On" : "🔍 Preview Off"}
      </button>
    </div>
  </div>

  <div class="row">
    <input type="text" bind:value={imagePath} placeholder="Path to disk image or /dev/sda..." disabled={busy} />
    <button onclick={loadMft} disabled={busy || !imagePath} class="btn-primary">Load</button>
  </div>

  <div class="workspace-split">
    <div class="table-container">
      {#if entries.length}
        <div class="table" style="--row-height:{densityRows[density]}">
          <div class="thead">
            <span class="col-name sortable" onclick={() => sortBy("filename")} title="Sort by name">Filename{sortIndicator("filename")}</span>
            <span class="col-rec sortable" onclick={() => sortBy("recordNumber")} title="Sort by record">Record{sortIndicator("recordNumber")}</span>
            <span class="col-size sortable" onclick={() => sortBy("fileSize")} title="Sort by size" style="text-align:right">Size{sortIndicator("fileSize")}</span>
            <span class="col-date sortable" onclick={() => sortBy("siCreated")} title="Sort by date">
              Created{sortIndicator("siCreated")}
            </span>
            <span class="col-status">Status</span>
          </div>
          <div class="tbody">
            {#each entries.slice(0, 500) as e, i}
              <button class="trow" class:deleted={e.isDeleted} style={getRowStyle(i)}
                  onclick={() => selectFile(e)} oncontextmenu={() => selectFile(e)}>
                <span class="col-name">{e.isDirectory ? "📁" : "📄"} {e.filename}</span>
                <span class="col-rec mono">#{e.recordNumber}</span>
                <span class="col-size mono" style="text-align:right">{sizeStr(e.fileSize)}</span>
                <span class="col-date mono">{formatTime(e.siCreated || e.fnCreated || "—")}</span>
                <span class="col-status">{e.isDeleted ? "🗑️ Deleted" : "✅"}</span>
              </button>
            {/each}
          </div>
        </div>
      {:else if busy}
        <div class="empty"><span class="spinner">⏳</span> Loading entries...</div>
      {:else}
        <div class="empty">Load a disk image to browse files</div>
      {/if}
    </div>

    {#if showPreview && previewFile}
      <div class="preview-split">
        <div class="preview-header">
          <span class="preview-filename">{previewFile}</span>
          <button class="close-btn" onclick={() => { previewFile = null; }}>✕</button>
        </div>
        <div class="preview-body">
          <FilePreview filePath={imagePath} bind:busy bind:msg {timeoutPromise} />
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
.file-browser { display:flex; flex-direction:column; height:100%; }
.toolbar { display:flex; align-items:center; justify-content:space-between; margin-bottom:12px; }
.toolbar h3 { margin:0; font-size:15px; }
.toolbar-right { display:flex; gap:4px; }
.preview-toggle { padding:3px 10px; background:transparent; border:1px solid var(--border); border-radius:6px; color:var(--text-secondary); cursor:pointer; font-size:11px; }
.preview-toggle.active { background:rgba(59,130,246,0.12); border-color:var(--primary); color:var(--primary); }
.row { display:flex; gap:8px; margin-bottom:12px; }
input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:8px 12px; flex:1; font-size:13px; }
.btn-primary { padding:8px 16px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; font-size:13px; }
.btn-primary:disabled { opacity:0.4; cursor:default; }

.workspace-split { display:flex; flex:1; gap:8px; overflow:hidden; }
.table-container { flex:1; overflow-y:auto; }
.preview-split { width:400px; min-width:300px; display:flex; flex-direction:column; border-left:1px solid var(--border); padding-left:8px; }
.preview-header { display:flex; align-items:center; justify-content:space-between; padding:6px 0; border-bottom:1px solid var(--border); margin-bottom:8px; }
.preview-filename { font-family:"SF Mono","Menlo",monospace; font-size:12px; color:#ccc; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.close-btn { background:none; border:none; color:var(--text-secondary); cursor:pointer; font-size:14px; padding:0 4px; }
.preview-body { flex:1; overflow-y:auto; }

.table { font-size:12px; }
.thead { display:grid; grid-template-columns:2fr 60px 80px 1fr 80px; padding:6px 8px; background:#111; border-radius:6px 6px 0 0; font-weight:600; position:sticky; top:0; z-index:10; }
.sortable { cursor:pointer; }
.sortable:hover { color:#e0e0e0; }
.tbody { }
.trow { display:grid; grid-template-columns:2fr 60px 80px 1fr 80px; padding:0 8px; border-bottom:1px solid rgba(255,255,255,0.04); align-items:center; cursor:pointer; width:100%; background:none; border:none; color:inherit; text-align:left; transition:background 0.1s; }
.trow:hover { background:rgba(59,130,246,0.08) !important; }
.deleted { opacity:0.5; text-decoration:line-through; }
.col-name { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.col-size, .col-rec, .col-date { overflow:hidden; text-overflow:ellipsis; }
.mono { font-family:"SF Mono","Menlo","Cascadia Code",monospace; }
.col-status { font-size:11px; }
.spinner { display:inline-block; animation:spin 1s linear infinite; }
@keyframes spin { to { transform:rotate(360deg); } }
.empty { display:flex; align-items:center; justify-content:center; height:200px; color:var(--text-secondary); font-size:14px; gap:6px; }
</style>
