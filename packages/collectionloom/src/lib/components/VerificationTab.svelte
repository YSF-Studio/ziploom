<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  let filePath = $state("");
  let expectedHash = $state("");
  let algorithm = $state("sha256");
  let busy = $state(false);
  let result = $state(null);
  let msg = $state("");

  $effect(() => {
    if (msg && !msg.startsWith("❌")) {
      const t = setTimeout(() => msg = "", 8000);
      return () => clearTimeout(t);
    }
  });

  async function browseFile() {
    try {
      const p = await open({ directory: false, multiple: false });
      if (p) filePath = p;
    } catch (e) { msg = `❌ ${String(e)}`; }
  }

  async function doVerify() {
    if (!filePath || !expectedHash) {
      msg = "❌ Select a file and enter the expected hash";
      return;
    }
    setBusy(true);
    result = null;
    try {
      result = await invoke("verify_hash", { path: filePath, expectedHash, algorithm });
    } catch (e) {
      msg = `❌ ${typeof e === 'string' ? e : String(e)}`;
    }
    setBusy(false);
  }

  function sizeStr(bytes) {
    if (!bytes) return "—";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes/1024).toFixed(1)} KB`;
    return `${(bytes/1048576).toFixed(1)} MB`;
  }
</script>

<div>
  <h3>🔐 Hash Verification</h3>
  <p class="note">Verify file integrity by comparing computed hash against expected value. Supports SHA-256, SHA-1, and MD5.</p>

  <div class="row">
    <label>Algorithm:
      <select bind:value={algorithm} disabled={busy}>
        <option value="sha256">SHA-256</option>
        <option value="sha1">SHA-1</option>
        <option value="md5">MD5</option>
      </select>
    </label>
  </div>

  <div class="row">
    <label>File:</label>
    <div class="file-row">
      <input type="text" bind:value={filePath} disabled={busy} placeholder="/path/to/evidence.dd" />
      <button class="btn-ghost" onclick={browseFile} disabled={busy}>📂</button>
    </div>
  </div>

  <div class="row">
    <label>Expected Hash:
      <input type="text" bind:value={expectedHash} disabled={busy}
        placeholder="e.g. a1b2c3d4..." class="hash-input" />
    </label>
  </div>

  {#if msg}
    <div class="result-card" class:error={msg.startsWith("❌")}>{msg}</div>
  {/if}

  <button class="btn-primary" onclick={doVerify} disabled={busy || !filePath || !expectedHash}>
    {busy ? '🔄 Verifying...' : '🔍 Verify'}
  </button>

  {#if result}
    <div class="result-card" class:match={result.matched} class:mismatch={!result.matched}>
      <span class="result-icon">{result.matched ? '✅' : '❌'}</span>
      <div>
        <strong>{result.matched ? 'MATCH — Hash verified!' : 'MISMATCH — Hash does not match!'}</strong><br />
        <div class="hash-compare">
          <div>
            <span class="hash-label">Expected:</span>
            <code>{result.expected}</code>
          </div>
          <div>
            <span class="hash-label">Actual:</span>
            <code class:good={result.matched} class:bad={!result.matched}>{result.actual}</code>
          </div>
        </div>
        <span class="muted">{result.algorithm.toUpperCase()} | {sizeStr(result.size)}</span>
      </div>
    </div>
  {/if}
</div>

<style>
  h3 { margin:0 0 8px; font-size:16px; }
  .note { font-size:11px; color:var(--text-secondary); margin: 0 0 20px; }
  .row { margin-bottom:12px; }
  label { font-size:13px; color:var(--text-secondary); display:flex; align-items:center; gap:6px; }
  input, select { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:6px 10px; font-size:13px; }
  input { width:100%; }
  .hash-input { font-family: monospace; }
  select { }
  .file-row { display:flex; gap:4px; flex:1; }
  .btn-ghost { background:transparent; border:1px solid var(--border); border-radius:6px; color:var(--text-secondary); cursor:pointer; padding:6px 10px; font-size:14px; }
  .btn-ghost:hover { border-color:var(--primary); }
  .btn-primary { padding:10px 24px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; margin-top:4px; }
  .btn-primary:disabled { opacity:0.4; cursor:default; }

  .result-card {
    margin-top:16px; padding:12px 16px; border-radius:8px; font-size:13px;
    display:flex; align-items:flex-start; gap:10px;
  }
  .result-card.match { background:rgba(34,197,94,0.1); border:1px solid var(--success); }
  .result-card.mismatch { background:rgba(239,68,68,0.1); border:1px solid var(--danger); }
  .result-card.error { background:rgba(239,68,68,0.1); border:1px solid var(--danger); color:var(--danger); }
  .result-icon { font-size:20px; flex-shrink:0; }
  .hash-compare { margin: 8px 0; }
  .hash-compare div { margin: 2px 0; }
  .hash-label { font-size:10px; color:var(--text-muted); margin-right:6px; }
  code { font-family:monospace; font-size:11px; word-break:break-all; }
  code.good { color:var(--success); }
  code.bad { color:var(--danger); }
  .muted { color:var(--text-muted); font-size:11px; }
</style>
