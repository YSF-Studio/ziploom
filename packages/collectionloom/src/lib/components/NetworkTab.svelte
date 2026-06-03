<script>
import { invoke } from "@tauri-apps/api/core";
let { setBusy, setMsg, timeoutPromise } = $props();
let interfaces = $state([]);
let iface = $state("");
let bpf = $state("");
let outFile = $state("/mnt/evidence/capture.pcapng");
let capturing = $state(false);

async function listIface() {
  setBusy(true);
  try { interfaces = await timeoutPromise(invoke("list_interfaces"), 5000); } catch(e) {}
  setBusy(false);
}
async function startCapture() {
  capturing = true;
  try {
    const r = await timeoutPromise(invoke("start_network_capture", { interface: iface, bpfFilter: bpf || null, outputFile: outFile }), 10000);
    setMsg(r);
  } catch(e) {}
  capturing = false;
}
async function stopCapture() {
  await invoke("cancel_network_capture");
  capturing = false;
}
$effect(() => { listIface(); });
</script>

<div>
  <h3>🌐 Network Capture</h3>
  <div class="row"><label>Interface: <select bind:value={iface}><option value="">-- Select --</option>{#each interfaces as i}<option value={i}>{i}</option>{/each}</select></label></div>
  <div class="row"><label>BPF Filter: <input type="text" bind:value={bpf} placeholder="not port 22" /></label></div>
  <div class="row"><label>Output: <input type="text" bind:value={outFile} /></label></div>
  {#if !capturing}
    <button onclick={startCapture} class="btn-primary" disabled={!iface}>▶ Start Capture</button>
  {:else}
    <button onclick={stopCapture} class="btn-danger">■ Stop</button>
  {/if}
</div>
<style>
h3 { margin:0 0 16px; font-size:16px; }
.row { margin-bottom:10px; }
label { font-size:13px; display:flex; align-items:center; gap:6px; }
input, select { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:6px 10px; width:100%; }
.btn-primary, .btn-danger { padding:10px 24px; color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary { background:var(--primary); }
.btn-danger { background:var(--danger); }
</style>