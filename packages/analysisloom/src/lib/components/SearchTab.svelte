<script>
import { invoke } from "@tauri-apps/api/core";
let { activeCase, busy, msg, timeoutPromise } = $props();
let query = $state("");
let results = $state([]);

async function search() {
  if (!query) return;
  busy = true;
  try {
    results = await timeoutPromise(invoke("keyword_search", { caseId: activeCase?.id, query }), 60000);
    msg = `✅ ${results.length} matches found`;
  } catch(e) { msg = `❌ ${typeof e === "string" ? e : String(e)}`; }
  busy = false;
}
</script>

<div>
  <h3>🔎 Keyword Search</h3>
  <div class="row">
    <input type="text" bind:value={query} placeholder="password|secret|key|token" disabled={busy} />
    <button onclick={search} disabled={busy||!query||!activeCase} class="btn-primary">Search</button>
  </div>
  {#if results.length}
  <div class="results">
    {#each results as r}
      <div class="r"><span class="file">{r.filePath}</span><span class="offset">@{r.offset}</span><span>{r.context}</span></div>
    {/each}
  </div>
  {/if}
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.row { display:flex; gap:8px; margin-bottom:12px; }
input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:8px 12px; flex:1; }
.btn-primary { padding:8px 16px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.results { margin-top:12px; font-size:12px; }
.r { display:flex; gap:10px; padding:4px 0; border-bottom:1px solid var(--border); }
.file { font-weight:600; }
.offset { color:var(--text-secondary); font-family:monospace; }
</style>