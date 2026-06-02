<script>
let { activeCase, busy, msg, timeoutPromise } = $props();
let events = $state([]);

async function loadTimeline() {
  busy = true;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    events = await timeoutPromise(invoke("get_timeline", { caseId: activeCase?.id }), 30000);
  } catch(e) {}
  busy = false;
}
</script>

<div>
  <h3>📊 Timeline Analysis</h3>
  <button onclick={loadTimeline} disabled={busy||!activeCase} class="btn-primary">Load Timeline</button>
  {#if events.length}
  <div class="timeline">
    {#each events.slice(0, 100) as evt}
      <div class="event"><span class="ts">{evt.timestamp}</span><span class="type">{evt.eventType}</span><span>{evt.filePath}</span></div>
    {/each}
  </div>
  {:else if !busy && activeCase}
    <p class="empty">No timeline events yet. Parse an NTFS image first.</p>
  {/if}
</div>
<style>
h3 { margin:0 0 16px; font-size:16px; }
.btn-primary { padding:8px 16px; background:var(--primary); color:white; border:none; border-radius:8px; cursor:pointer; font-weight:600; }
.timeline { margin-top:16px; font-size:12px; }
.event { display:flex; gap:12px; padding:4px 0; border-bottom:1px solid var(--border); }
.ts { color:var(--text-secondary); white-space:nowrap; min-width:140px; }
.type { font-weight:600; min-width:80px; }
.empty { margin-top:16px; color:var(--text-secondary); font-size:13px; }
</style>