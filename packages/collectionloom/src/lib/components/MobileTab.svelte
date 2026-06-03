<script>
import { invoke } from "@tauri-apps/api/core";
let { setBusy, setMsg, timeoutPromise } = $props();
let androidDevices = $state([]);
let iosDevices = $state([]);
let outputPath = $state("/mnt/evidence/");

async function scanAndroid() {
  setBusy(true);
  try { androidDevices = await timeoutPromise(invoke("list_android_devices"), 10000); } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
async function scanIos() {
  setBusy(true);
  try { iosDevices = await timeoutPromise(invoke("list_ios_devices"), 10000); } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
async function backupAndroid(id) {
  setBusy(true);
  try {
    const r = await timeoutPromise(invoke("adb_backup", { deviceId: id, output: outputPath + "android_backup.ab" }), 300000);
    setMsg(`✅ ${r}`);
  } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
</script>

<div>
  <h3>📱 Mobile Triage</h3>
  <div class="cols">
    <div class="col">
      <h4>Android</h4>
      <button onclick={scanAndroid} disabled={busy} class="btn-sm">🔍 Scan ADB</button>
      {#each androidDevices as d}
        <div class="device">{d.model} ({d.id}) <button onclick={() => backupAndroid(d.id)} class="btn-sm">💾 Backup</button></div>
      {/each}
    </div>
    <div class="col">
      <h4>iOS</h4>
      <button onclick={scanIos} disabled={busy} class="btn-sm">🔍 Scan idevice</button>
      {#each iosDevices as d}
        <div class="device">{d.model} ({d.id}) <button class="btn-sm">💾 Backup</button></div>
      {/each}
    </div>
  </div>
  <p class="note">⚠️ Faraday bag reminder: isolate mobile devices before acquisition</p>
</div>
<style>
h3 { margin:0 0 16px; font-size:16px; }
.cols { display:grid; grid-template-columns:1fr 1fr; gap:20px; }
.col { background:#1a1a1a; border:1px solid var(--border); border-radius:8px; padding:12px; }
h4 { margin:0 0 8px; font-size:13px; }
.device { font-size:12px; padding:6px 0; border-bottom:1px solid var(--border); display:flex; justify-content:space-between; align-items:center; }
.btn-sm { padding:4px 8px; background:var(--border); color:#e0e0e0; border:none; border-radius:4px; cursor:pointer; font-size:11px; }
.note { font-size:11px; color:var(--warn); margin-top:16px; }
</style>