<script>
let provider = $state("aws");
let region = $state("us-east-1");
let volumeId = $state("");
let accessKey = $state("");
let secretKey = $state("");
let msg = $state("");
</script>

<div>
  <h3>☁️ Cloud Snapshot</h3>
  <p class="note">API keys held in RAM only — never written to disk</p>
  <div class="row">
    <label>Provider: <select bind:value={provider}>
      <option value="aws">AWS</option><option value="azure">Azure</option><option value="gcp">GCP</option>
    </select></label>
  </div>
  <div class="row"><label>Region: <input type="text" bind:value={region} /></label></div>
  <div class="row"><label>Volume/Disk ID: <input type="text" bind:value={volumeId} /></label></div>
  <div class="row"><label>Access Key: <input type="password" bind:value={accessKey} /></label></div>
  <div class="row"><label>Secret Key: <input type="password" bind:value={secretKey} /></label></div>
  <button class="btn-primary">Create Snapshot</button>
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.row { margin-bottom:10px; }
label { font-size:13px; display:flex; align-items:center; gap:6px; }
input, select { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:6px 10px; width:300px; }
.btn-primary { padding:10px 24px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; margin-top:12px; }
.note { font-size:11px; color:var(--text-secondary); }
</style>