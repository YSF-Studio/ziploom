<script>
import { invoke } from "@tauri-apps/api/core";
let { sharedState, busy, setBusy, setMsg, timeoutPromise } = $props();

let disks = $state([]);
let selectedDisk = $state("");
let destPath = $state("/mnt/evidence/image.dd");
let splitSize = $state("0");
let shouldVerify = $state(true);
let progress = $state({ percent: 0, status: "Idle", bytesProcessed: 0, totalBytes: 0 });
let collBusy = $state(false);
let pollId = $state(null);

async function listDisks() {
  setBusy(true);
  try {
    disks = await timeoutPromise(invoke("list_disks"), 15000);
  } catch(e) {
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT") setMsg(`\u274c ${err}`);
  }
  setBusy(false);
}

async function startImaging() {
  if (!selectedDisk || !destPath) { setMsg("\u26a0\ufe0f Select a disk and destination"); return; }
  collBusy = true;
  try {
    await timeoutPromise(invoke("start_disk_imaging", { source: selectedDisk, destination: destPath, splitSizeMb: parseInt(splitSize) || 0, verify: shouldVerify }), 5000);
    // Poll progress
    pollId = setInterval(async () => {
      try {
        const p = await invoke("get_imaging_progress");
        progress = p;
        if (p.isDone) { clearInterval(pollId); collBusy = false; setMsg(p.error ? `\u274c ${p.error}` : "\u2705 Imaging complete!"); }
      } catch(e) { clearInterval(pollId); collBusy = false; }
    }, 500);
  } catch(e) {
    collBusy = false;
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT") setMsg(`\u274c ${err}`);
  }
}

async function cancelImaging() {
  await invoke("cancel_imaging");
  if (pollId) clearInterval(pollId);
  collBusy = false;
  progress.status = "Cancelled";
}

// Load on mount
$effect(() => { listDisks(); });
</script>

<div>
  <h3>💿 Disk Acquisition</h3>
  
  <div class="row">
    <label>Source Device:
      <select bind:value={selectedDisk} disabled={collBusy||busy}>
        <option value="">-- Select disk --</option>
        {#each disks as disk}
          <option value={disk.device}>{disk.device} ({disk.model}) — {(disk.sizeBytes/1e9).toFixed(1)} GB {disk.isSsd ? "SSD" : "HDD"}</option>
        {/each}
      </select>
    </label>
    <button onclick={listDisks} class="btn-sm">🔄 Refresh</button>
  </div>

  <div class="row">
    <label>Destination: <input type="text" bind:value={destPath} disabled={collBusy} placeholder="/mnt/evidence/image.dd" /></label>
  </div>

  <div class="options">
    <label>Split (MB): <input type="number" bind:value={splitSize} disabled={collBusy} placeholder="0=no split" style="width:80px" /></label>
    <label><input type="checkbox" bind:checked={shouldVerify} disabled={collBusy} /> Verify after write</label>
  </div>

  <div class="actions">
    {#if !collBusy}
      <button onclick={startImaging} class="btn-primary" disabled={!selectedDisk}>▶️ Start Collection</button>
    {:else}
      <button onclick={cancelImaging} class="btn-danger">■ Stop</button>
    {/if}
  </div>

  {#if collBusy || progress.bytesProcessed > 0}
  <div class="progress-bar">
    <div class="fill" style="width:{progress.percent}%"></div>
  </div>
  <div class="progress-info">
    <span>{progress.percent.toFixed(1)}%</span>
    <span>{progress.status}</span>
    <span>{(progress.bytesProcessed / 1e9).toFixed(2)} GB</span>
  </div>
  {/if}
</div>

<style>
h3 { margin: 0 0 16px; font-size: 16px; }
.row { display: flex; gap: 10px; align-items: center; margin-bottom: 12px; }
label { font-size: 13px; display: flex; align-items: center; gap: 6px; }
select, input { background: #1a1a1a; color: #e0e0e0; border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; font-size: 13px; width: 100%; }
.options { display: flex; gap: 16px; margin: 12px 0; font-size: 13px; }
.actions { margin: 16px 0; }
.btn-primary { padding: 10px 24px; background: var(--primary); color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 600; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-danger { padding: 10px 24px; background: var(--danger); color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; }
.btn-sm { padding: 5px 10px; background: var(--border); color: #e0e0e0; border: none; border-radius: 6px; cursor: pointer; font-size: 12px; }
.progress-bar { height: 8px; background: #2a2a2a; border-radius: 4px; margin: 12px 0; overflow: hidden; }
.fill { height: 100%; background: var(--primary); border-radius: 4px; transition: width 0.3s; }
.progress-info { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-secondary); }
</style>