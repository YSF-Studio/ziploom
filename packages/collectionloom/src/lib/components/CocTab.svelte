<script>
import { invoke } from "@tauri-apps/api/core";
let { state, setBusy, setMsg, timeoutPromise } = $props();
let evidenceId = $state("");
let caseName = $state("");
let operator = $state("Yusuf Shalahuddin");
let device = $state("");
let actions = $state([]);

async function createCoc() {
  setBusy(true);
  try {
    evidenceId = await timeoutPromise(invoke("create_chain_of_custody", { caseName, operator, sourceDevice: device }), 5000);
    actions = [{ timestamp: new Date().toISOString(), action: "CoC created", details: `Evidence ${evidenceId}`, hash: null }];
    setMsg(`✅ Chain of custody created: ${evidenceId}`);
  } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
async function addAction(act, det) {
  actions = [...actions, { timestamp: new Date().toISOString(), action: act, details: det, hash: null }];
}
async function generatePdf() {
  setBusy(true);
  try {
    const path = await timeoutPromise(invoke("generate_coc_report", { evidenceId }), 15000);
    setMsg(`✅ PDF report saved to ${path}`);
  } catch(e) { setMsg(`❌ ${typeof e === "string" ? e : String(e)}`); }
  setBusy(false);
}
</script>

<div>
  <h3>📋 Chain of Custody</h3>
  <div class="row"><label>Case: <input type="text" bind:value={caseName} /></label></div>
  <div class="row"><label>Operator: <input type="text" bind:value={operator} /></label></div>
  <div class="row"><label>Source Device: <input type="text" bind:value={device} placeholder="/dev/sda" /></label></div>
  
  {#if !evidenceId}
    <button onclick={createCoc} class="btn-primary" disabled={!caseName||!device}>📋 Create Chain of Custody</button>
  {:else}
    <div class="evidence-id">Evidence ID: <strong>{evidenceId}</strong></div>
    
    <div class="actions-log">
      <h4>Action Log</h4>
      {#each actions as a, i}
        <div class="log-entry">
          <span class="time">{a.timestamp}</span>
          <span class="action">{a.action}</span>
          <span class="detail">{a.details}</span>
        </div>
      {/each}
    </div>

    <div class="add-action">
      <input type="text" placeholder="Action (e.g. imaging_start)" id="newAction" />
      <input type="text" placeholder="Details" id="newDetails" />
      <button onclick={() => { let a=document.getElementById('newAction').value; let d=document.getElementById('newDetails').value; addAction(a,d); }} class="btn-sm">+ Add</button>
    </div>

    <button onclick={generatePdf} class="btn-primary" style="margin-top:16px">📄 Generate PDF Report</button>
  {/if}
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.row { margin-bottom:10px; }
label { font-size:13px; display:flex; align-items:center; gap:6px; }
input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:6px 10px; width:300px; }
.btn-primary { padding:10px 24px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary:disabled { opacity:0.5; }
.btn-sm { padding:4px 10px; background:var(--border); color:#e0e0e0; border:none; border-radius:4px; cursor:pointer; font-size:11px; }
.evidence-id { padding:10px; background:#1a2e1a; border:1px solid var(--success); border-radius:8px; margin:12px 0; }
.actions-log { margin:16px 0; max-height:200px; overflow-y:auto; }
h4 { font-size:13px; margin:0 0 8px; }
.log-entry { display:flex; gap:10px; padding:4px 0; font-size:11px; border-bottom:1px solid var(--border); }
.time { color:var(--text-secondary); white-space:nowrap; }
.action { font-weight:600; min-width:120px; }
.add-action { display:flex; gap:6px; margin-top:10px; }
.add-action input { width:150px; }
</style>