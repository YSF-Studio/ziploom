<script>
import { invoke } from "@tauri-apps/api/core";
let { activeCase, busy, msg, timeoutPromise } = $props();
let imagePath = $state("");
let outputDir = $state("/tmp/carved");
let result = $state(null);
let progress = $state({ percent: 0, status: "Idle" });
let pollId = $state(null);

async function carve() {
  busy = true; result = null;
  try {
    await timeoutPromise(invoke("start_carving", { imagePath, outputDir }), 5000);
    pollId = setInterval(async () => {
      try {
        const p = await invoke("get_carving_progress");
        progress = p;
        if (p.isDone) { clearInterval(pollId); busy = false; msg = `✅ ${p.status}`; }
      } catch(e) { clearInterval(pollId); busy = false; }
    }, 500);
  } catch(e) { busy = false; }
}
async function cancel() { await invoke("cancel_carving"); if (pollId) clearInterval(pollId); busy = false; }
</script>

<div>
  <h3>🔍 File Carving</h3>
  <div class="row"><input type="text" bind:value={imagePath} placeholder="Disk image path" /><input type="text" bind:value={outputDir} placeholder="Output directory" /></div>
  {#if !busy}<button onclick={carve} disabled={!imagePath} class="btn-primary">▶ Start Carving</button>
  {:else}<button onclick={cancel} class="btn-danger">■ Stop</button>{/if}
  {#if progress.percent > 0}
    <div class="progress-bar"><div class="fill" style="width:{progress.percent}%"></div></div>
    <p class="info">{progress.status}</p>
  {/if}
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.row { display:flex; gap:8px; margin-bottom:12px; }
input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:8px 12px; flex:1; }
.btn-primary,.btn-danger { padding:8px 16px; color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary { background:var(--primary); }
.btn-danger { background:var(--danger); }
.progress-bar { height:8px; background:#2a2a2a; border-radius:4px; margin:12px 0; overflow:hidden; }
.fill { height:100%; background:var(--primary); border-radius:4px; transition:width 0.3s; }
.info { font-size:12px; color:var(--text-secondary); }
</style>