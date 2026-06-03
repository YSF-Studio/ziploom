<script>
import { invoke } from "@tauri-apps/api/core";
let { state, setBusy, setMsg, timeoutPromise } = $props();

let report = $state(null);

async function scanNow() {
  setBusy(true);
  try {
    report = await timeoutPromise(invoke("scan_encryption"), 30000);
    if (report.hasFde) {
      setMsg(`\u26a0\ufe0f Full Disk Encryption detected — capture RAM before shutdown!`);
    } else {
      setMsg("\u2705 No full disk encryption detected");
    }
  } catch(e) {
    const err = typeof e === "string" ? e : String(e);
    if (err !== "TIMEOUT") setMsg(`\u274c ${err}`);
  }
  setBusy(false);
}
</script>

<div>
  <h3>\ud83d\udd10 Encryption Detection</h3>
  <p class="desc">Pre-acquisition triage — detects FDE, TPM, and Secure Boot status.</p>

  <button onclick={scanNow} class="btn-primary" disabled={busy}>
    {busy ? "\u23f3 Scanning..." : "\ud83d\udd0d Scan All Drives"}
  </button>

  {#if report}
  <div class="report">
    <div class="summary card {report.hasFde ? 'danger' : 'success'}">
      <strong>Platform:</strong> {report.platform} &nbsp;
      <strong>FDE:</strong> {report.hasFde ? "\u26a0\ufe0f YES" : "\u2705 No"} &nbsp;
      <strong>TPM:</strong> {report.tpmPresent ? "\u2705 Present" : "\u274c Not found"} &nbsp;
      <strong>Secure Boot:</strong> {report.secureBoot || "Unknown"}
    </div>

    {#if report.fdeType}
    <div class="card">
      <h4>Encryption Type</h4>
      <pre>{JSON.stringify(report.fdeType, null, 2)}</pre>
    </div>
    {/if}

    {#if report.recommendations?.length}
    <div class="card warn">
      <h4>\ud83d\udcd6 Recommendations</h4>
      {#each report.recommendations as rec}
        <p class="rec">\u2022 {rec}</p>
      {/each}
    </div>
    {/if}
  </div>
  {/if}
</div>

<style>
h3 { margin: 0 0 8px; font-size: 16px; }
.desc { font-size: 12px; color: var(--text-secondary); margin-bottom: 16px; }
.card { padding: 12px; border-radius: 8px; margin: 10px 0; border: 1px solid var(--border); background: #1a1a1a; }
.card.danger { border-color: var(--danger); background: #2e1a1a; }
.card.success { border-color: var(--success); background: #1a2e1a; }
.card.warn { border-color: var(--warn); }
h4 { margin: 0 0 6px; font-size: 13px; }
pre { font-size: 11px; overflow-x: auto; }
.rec { font-size: 12px; margin: 4px 0; }
.btn-primary { padding: 10px 24px; background: var(--primary); color: white; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 600; }
.btn-primary:disabled { opacity: 0.5; }
</style>