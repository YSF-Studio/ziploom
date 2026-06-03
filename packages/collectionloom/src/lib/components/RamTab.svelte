<script>
import { invoke } from "@tauri-apps/api/core";
let { state, setBusy, setMsg, timeoutPromise } = $props();
let tools = $state([]);
let selectedTool = $state("");
let outputPath = $state("/mnt/evidence/ram_capture.lime");
let compress = $state(true);
let progress = $state(null);
let ramSize = $state(null);

async function listTools() {
  setBusy(true);
  try { tools = await timeoutPromise(invoke("list_ram_tools"), 5000); } catch(e) {}
  try { ramSize = await timeoutPromise(invoke("get_ram_size"), 5000); } catch(e) {}
  setBusy(false);
}
async function capture() {
  setBusy(true);
  try {
    const result = await timeoutPromise(invoke("capture_ram", { tool: selectedTool, output: outputPath, compress }), 120000);
    setMsg(`✅ ${result}`);
  } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
$effect(() => { listTools(); });
</script>

<div>
  <h3>🧠 RAM Capture</h3>
  {#if ramSize}<p class="info">System RAM: {(ramSize/1e9).toFixed(1)} GB</p>{/if}
  <div class="row">
    <label>Tool: <select bind:value={selectedTool} disabled={busy}>
      <option value="">-- Select --</option>
      {#each tools as tool}<option value={tool}>{tool}</option>{/each}
    </select></label>
  </div>
  <div class="row">
    <label>Output: <input type="text" bind:value={outputPath} disabled={busy} /></label>
    <label><input type="checkbox" bind:checked={compress} disabled={busy} /> Compress</label>
  </div>
  <button onclick={capture} class="btn-primary" disabled={busy||!selectedTool}>▶ Capture RAM</button>
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.info { font-size:12px; color:var(--text-secondary); margin-bottom:10px; }
.row { display:flex; gap:10px; align-items:center; margin-bottom:12px; }
select, input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:6px 10px; }
.btn-primary { padding:10px 24px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary:disabled { opacity:0.5; }
</style>