<script>
import { invoke } from "@tauri-apps/api/core";
let { activeCase, busy, msg, timeoutPromise } = $props();
let imagePath = $state("");
let entries = $state([]);
let preview = $state(null);

async function loadMft() {
  if (!imagePath) return;
  busy = true;
  try {
    entries = await timeoutPromise(invoke("parse_mft", { imagePath }), 60000);
    msg = `✅ ${entries.length} entries loaded`;
  } catch(e) { msg = `❌ ${typeof e === "string" ? e : String(e)}`; }
  busy = false;
}
</script>

<div>
  <h3>🗂️ File Browser (NTFS)</h3>
  <div class="row">
    <input type="text" bind:value={imagePath} placeholder="Path to disk image or /dev/sda..." disabled={busy} />
    <button onclick={loadMft} disabled={busy||!imagePath} class="btn-primary">Load</button>
  </div>

  {#if entries.length}
  <div class="table">
    <div class="header"><span>Filename</span><span>Record</span><span>Size</span><span>Created</span><span>Deleted</span></div>
    {#each entries.slice(0, 200) as e}
      <div class="row" class:deleted={e.isDeleted}>
        <span>{e.isDirectory ? "📁" : "📄"} {e.filename}</span>
        <span>#{e.recordNumber}</span>
        <span>{(e.fileSize/1024).toFixed(1)} KB</span>
        <span>{e.siCreated || e.fnCreated || "—"}</span>
        <span>{e.isDeleted ? "🗑️" : ""}</span>
      </div>
    {/each}
  </div>
  {/if}
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.row { display:flex; gap:8px; margin-bottom:12px; }
input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:8px 12px; flex:1; }
.btn-primary { padding:8px 16px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.table { font-size:12px; overflow-x:auto; }
.header { display:grid; grid-template-columns:2fr 80px 80px 1fr 60px; padding:6px 10px; background:#1a1a1a; border-radius:6px; font-weight:600; }
.table .row { display:grid; grid-template-columns:2fr 80px 80px 1fr 60px; padding:4px 10px; border-bottom:1px solid var(--border); margin:0; }
.deleted { opacity:0.5; text-decoration:line-through; }
</style>