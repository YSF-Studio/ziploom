<script>
  import { invoke } from "@tauri-apps/api/core";

  let provider = $state("aws");
  let region = $state("us-east-1");
  let resourceId = $state("");
  let accessKey = $state("");
  let secretKey = $state("");
  let busy = $state(false);
  let msg = $state("");
  let result = $state(null);
  let resultRaw = $state("");

  $effect(() => {
    if (msg && !msg.startsWith("❌")) {
      const t = setTimeout(() => msg = "", 8000);
      return () => clearTimeout(t);
    }
  });

  function placeholderText() {
    if (provider === "aws") return "vol-xxxxxxxxxxxx";
    if (provider === "azure") return "sub-id|rg-name|disk-name";
    if (provider === "gcp") return "project-id|zone|disk-name";
    return "";
  }

  async function doCreateSnapshot() {
    if (!resourceId || !accessKey || !secretKey) {
      msg = "❌ All fields are required";
      return;
    }
    setBusy(true);
    result = null;
    resultRaw = "";
    msg = "";
    try {
      const res = await invoke("create_cloud_snapshot", {
        provider,
        region,
        resourceId,
        accessKey,
        secretKey,
      });
      result = res;
      resultRaw = JSON.stringify(res, null, 2);
    } catch (e) {
      const err = typeof e === 'string' ? e : String(e);
      // AWS returns raw response even on error — show it
      if (err.includes("<?xml") || err.includes("<?XML") || err.includes("<Create")) {
        resultRaw = err;
        result = { provider, response: "(see raw response below)" };
      } else {
        msg = `❌ ${err}`;
      }
    }
    setBusy(false);
  }
</script>

<div>
  <h3>☁️ Cloud Snapshot</h3>
  <p class="note">API keys held in RAM only — never written to disk</p>

  <div class="row">
    <label>Provider:
      <select bind:value={provider} disabled={busy}>
        <option value="aws">AWS — Create EBS Snapshot</option>
        <option value="azure">Azure — Create Disk Snapshot</option>
        <option value="gcp">GCP — Create Persistent Disk Snapshot</option>
      </select>
    </label>
  </div>

  <div class="row">
    <label>Region:
      <input type="text" bind:value={region} disabled={busy} />
    </label>
  </div>

  <div class="row">
    <label>Resource ID:
      <input type="text" bind:value={resourceId} disabled={busy} placeholder={placeholderText()} />
    </label>
    <span class="hint">
      {provider === "aws" ? "AWS: Volume ID (vol-...)" : provider === "azure" ? "Azure: subscription|resourceGroup|diskName" : "GCP: project|zone|diskName"}
    </span>
  </div>

  <div class="row">
    <label>Access Key / Client ID:
      <input type="password" bind:value={accessKey} disabled={busy} />
    </label>
  </div>

  <div class="row">
    <label>Secret Key / Token:
      <input type="password" bind:value={secretKey} disabled={busy} />
    </label>
  </div>

  {#if msg}
    <div class="result-card" class:error={msg.startsWith("❌")}>{msg}</div>
  {/if}

  <button class="btn-primary" onclick={doCreateSnapshot} disabled={busy}>
    {busy ? "🔄 Creating Snapshot..." : "📸 Create Snapshot"}
  </button>

  {#if result}
    <div class="result-card success">
      <strong>✅ Snapshot Request Sent</strong><br />
      <span class="muted">Provider: {result.provider}</span>
    </div>
    {#if resultRaw}
      <pre class="raw-response">{resultRaw}</pre>
    {/if}
  {/if}

  {#if busy}
    <div class="spinner">⏳ Contacting cloud provider... (may take 15-30s)</div>
  {/if}
</div>

<style>
  h3 { margin:0 0 8px; font-size:16px; }
  .row { margin-bottom:10px; }
  label { font-size:13px; display:flex; align-items:center; gap:6px; }
  input, select {
    background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border);
    border-radius:6px; padding:6px 10px; width:320px; font-size:13px;
  }
  input:disabled, select:disabled { opacity: 0.5; }
  .btn-primary {
    padding:10px 24px; background:var(--primary); color:white;
    border:none; border-radius:8px; cursor:pointer; font-weight:600; margin-top:12px;
    transition: filter 0.15s;
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .note { font-size:11px; color:var(--text-secondary); margin: 0 0 16px; }
  .hint { font-size:10px; color:var(--text-muted); display:block; margin-top:2px; margin-left:2px; }
  .result-card {
    margin-top:12px; padding:10px 14px; border-radius:8px; font-size:13px;
    background: rgba(34,197,94,0.1); border: 1px solid var(--success);
  }
  .result-card.error {
    background: rgba(239,68,68,0.1); border: 1px solid var(--danger); color: var(--danger);
  }
  .muted { color: var(--text-muted); font-size:11px; }
  .raw-response {
    margin-top:12px; padding:10px; background:#0a0a0a; border:1px solid var(--border);
    border-radius:6px; font-size:11px; font-family: var(--mono); max-height:300px;
    overflow:auto; white-space:pre-wrap; word-break:break-all; color: var(--text-secondary);
  }
  .spinner { margin-top:12px; font-size:13px; color: var(--primary); }
</style>
