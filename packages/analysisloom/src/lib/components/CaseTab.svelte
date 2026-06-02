<script>
import { invoke } from "@tauri-apps/api/core";
let { activeCase, busy, msg, timeoutPromise } = $props();
let cases = $state([]);
let newCaseName = $state("");

async function loadCases() {
  busy = true;
  try { cases = await timeoutPromise(invoke("list_cases"), 5000); } catch(e) {}
  busy = false;
}
async function createCase() {
  if (!newCaseName) return;
  busy = true;
  try {
    const c = await timeoutPromise(invoke("create_case", { name: newCaseName, operator: "Yusuf Shalahuddin" }), 5000);
    cases = [c, ...cases]; activeCase = c; newCaseName = "";
    msg = `✅ Case created: ${c.id}`;
  } catch(e) { msg = `❌ ${typeof e === "string" ? e : String(e)}`; }
  busy = false;
}
$effect(() => { loadCases(); });
</script>

<div>
  <h3>📁 Case Management</h3>
  <div class="new-case">
    <input type="text" bind:value={newCaseName} placeholder="Case name..." disabled={busy} />
    <button onclick={createCase} disabled={busy||!newCaseName} class="btn-primary">+ New Case</button>
  </div>
  <div class="case-list">
    {#each cases as c}
      <div class="case-card" class:active={activeCase?.id === c.id} onclick={() => activeCase = c}>
        <strong>{c.name}</strong>
        <span class="meta">{c.id} | {c.createdAt}</span>
        <span class="status">{c.status}</span>
      </div>
    {/each}
  </div>
  {#if !cases.length && !busy}<p class="empty">No cases yet. Create one to get started.</p>{/if}
</div>

<style>
h3 { margin:0 0 16px; font-size:16px; }
.new-case { display:flex; gap:8px; margin-bottom:16px; }
input { background:#1a1a1a; color:#e0e0e0; border:1px solid var(--border); border-radius:6px; padding:8px 12px; flex:1; }
.btn-primary { padding:8px 16px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.btn-primary:disabled { opacity:0.5; }
.case-list { display:flex; flex-direction:column; gap:6px; }
.case-card { padding:12px; border:1px solid var(--border); border-radius:8px; cursor:pointer; display:flex; align-items:center; gap:12px; }
.case-card:hover { background:#1a1a1a; }
.case-card.active { border-color:var(--primary); background:#1a1a2e; }
.meta { font-size:11px; color:var(--text-secondary); margin-left:auto; }
.status { font-size:11px; padding:2px 8px; border-radius:4px; background:#1a2e1a; }
</style>