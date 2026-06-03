<script>
import { invoke } from "@tauri-apps/api/core";
let { setBusy, setMsg, timeoutPromise } = $props();
let device = $state("");
let status = $state("Unknown");
let enabled = $state(false);

async function enable() {
  setBusy(true);
  try {
    await timeoutPromise(invoke("enable_write_blocker", { device }), 10000);
    enabled = true; status = "ACTIVE"; setMsg("✅ Write blocker enabled");
    // Verify
    try { await invoke("check_write_blocker", { device }); } catch{}
  } catch(e) {
    const err = typeof e === "string" ? e : String(e);
    setMsg(`❌ ${err}`);
  }
  setBusy(false);
}
async function disable() {
  setBusy(true);
  try {
    await timeoutPromise(invoke("disable_write_blocker", { device }), 10000);
    enabled = false; status = "Disabled"; setMsg("✅ Write blocker disabled");
  } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
</script>

<div>
  <h3>🛡️ Write Blocker</h3>
  <div class="row">
    <label>Device: <input type="text" bind:value={device} placeholder="/dev/sda" disabled={busy} /></label>
  </div>
  <div class="status" class:active={enabled}>{enabled ? "🟢 ACTIVE" : "⚫ Disabled"}</div>
  <div class="actions">
    <button onclick={enable} class="btn-primary" disabled={busy||!device}>🛡️ Enable</button>
    <button onclick={disable} class="btn-danger" disabled={busy||!enabled}>🔓 Disable</button>
  </div>
  <p class="note">Platform: {navigator.platform}</p>
</div>

<style>
h3 { margin: 0 0 16px; font-size: 16px; }
.row { margin-bottom: 12px; }
input { background: #1a1a1a; color: #e0e0e0; border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; width: 300px; }
.status { padding: 10px; border-radius: 8px; font-size: 14px; font-weight: 600; margin: 10px 0; background: #1a1a1a; border: 1px solid var(--border); }
.status.active { background: #1a2e1a; border-color: var(--success); color: var(--success); }
.actions { display: flex; gap: 10px; margin: 16px 0; }
.btn-primary, .btn-danger { padding: 10px 20px; color: white; border: none; border-radius: 8px; cursor: pointer; font-weight: 600; }
.btn-primary { background: var(--primary); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-danger { background: var(--danger); }
.note { font-size: 11px; color: var(--text-secondary); margin-top: 20px; }
</style>