<script>
  import { invoke } from "@tauri-apps/api/core";

  let { activeCase, busy, msg, timeoutPromise } = $props();

  let bookmarks = $state([]);
  let loading = $state(false);
  let showAdd = $state(false);
  let addFilePath = $state("");
  let addTag = $state("");
  let addNote = $state("");

  async function loadBookmarks() {
    if (!activeCase?.id) return;
    loading = true;
    try {
      bookmarks = await timeoutPromise(invoke("list_bookmarks", { caseId: activeCase.id }), 10000);
    } catch (e) {
      bookmarks = [];
    }
    loading = false;
  }

  async function doAddBookmark() {
    if (!addFilePath || !activeCase?.id) return;
    try {
      await invoke("add_bookmark", {
        caseId: activeCase.id,
        filePath: addFilePath,
        offset: 0,
        tag: addTag || null,
        note: addNote || null,
      });
      msg.set(`✅ Bookmark added: ${addFilePath}`);
      addFilePath = "";
      addTag = "";
      addNote = "";
      showAdd = false;
      await loadBookmarks();
    } catch (e) {
      msg.set(`❌ ${typeof e === "string" ? e : String(e)}`);
    }
  }

  async function doDeleteBookmark(id) {
    try {
      await invoke("delete_bookmark", { id });
      bookmarks = bookmarks.filter(b => b.id !== id);
      msg.set("✅ Bookmark deleted");
    } catch (e) {
      msg.set(`❌ ${typeof e === "string" ? e : String(e)}`);
    }
  }

  function copyToAdd(file) {
    addFilePath = file;
    showAdd = true;
  }

  // Reload when case changes
  $effect(() => {
    if (activeCase?.id) loadBookmarks();
  });
</script>

<div class="bookmark-tab">
  <div class="toolbar">
    <h3>🔖 Bookmarks</h3>
    <button class="btn-ghost" onclick={() => { showAdd = !showAdd; if (!showAdd) { addFilePath = ""; addTag = ""; addNote = ""; } }}>
      {showAdd ? "✕ Cancel" : "+ Add"}
    </button>
  </div>

  {#if !activeCase?.id}
    <div class="empty-state">
      <span class="icon">📂</span>
      <p>Open a case first from the Case Manager</p>
    </div>
  {:else}

    <!-- Add Bookmark Form -->
    {#if showAdd}
      <div class="card add-form">
        <div class="row">
          <label>File Path:</label>
          <input type="text" bind:value={addFilePath} placeholder="/path/to/evidence/file" />
        </div>
        <div class="row">
          <label>Tag:</label>
          <input type="text" bind:value={addTag} placeholder="e.g. suspicious, important, malware" />
        </div>
        <div class="row">
          <label>Note:</label>
          <textarea bind:value={addNote} placeholder="Optional note about this bookmark..." rows="2"></textarea>
        </div>
        <button class="btn-primary" onclick={doAddBookmark} disabled={!addFilePath}>
          ✅ Save Bookmark
        </button>
      </div>
    {/if}

    <!-- Bookmarks List -->
    {#if loading}
      <div class="loading-state"><span class="spinner">⏳</span> Loading bookmarks...</div>
    {:else if bookmarks.length > 0}
      <div class="bm-list">
        {#each bookmarks as bm}
          <div class="bm-card">
            <div class="bm-header">
              <span class="bm-file" title={bm.filePath}>{bm.filePath}</span>
              <button class="btn-delete" onclick={() => doDeleteBookmark(bm.id)}>✕</button>
            </div>
            <div class="bm-meta">
              {#if bm.tag}
                <span class="bm-tag">{bm.tag}</span>
              {/if}
              {#if bm.offset > 0}
                <span class="bm-offset">@ offset: 0x{bm.offset.toString(16).toUpperCase()}</span>
              {/if}
            </div>
            {#if bm.note}
              <p class="bm-note">{bm.note}</p>
            {/if}
            <span class="bm-time">{bm.createdAt || "—"}</span>
          </div>
        {/each}
      </div>
    {:else if !loading}
      <div class="empty-state">
        <span class="icon">🔖</span>
        <p>No bookmarks yet</p>
        <span class="hint">Add bookmarks to save important files and findings for quick reference.</span>
        <button class="btn-ghost" onclick={() => showAdd = true} style="margin-top:12px">+ Add Your First Bookmark</button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .bookmark-tab { display: flex; flex-direction: column; height: 100%; }
  .toolbar { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }
  .toolbar h3 { margin: 0; font-size: 16px; }
  .btn-ghost { padding: 6px 14px; background: transparent; border: 1px solid var(--border); border-radius: 6px; color: var(--text-secondary); cursor: pointer; font-size: 12px; }
  .btn-ghost:hover { border-color: var(--primary); color: var(--primary); }

  .card { background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 14px 16px; margin-bottom: 16px; }
  .add-form .row { margin-bottom: 10px; }
  .add-form label { display: block; font-size: 12px; color: var(--text-secondary); margin-bottom: 4px; }
  .add-form input, .add-form textarea { background: #1a1a1a; color: #e0e0e0; border: 1px solid var(--border); border-radius: 6px; padding: 8px 10px; font-size: 13px; width: 100%; box-sizing: border-box; }
  .add-form textarea { resize: vertical; font-family: inherit; }
  .btn-primary { padding: 8px 18px; background: var(--primary); color: white; border: none; border-radius: 8px; font-weight: 600; font-size: 13px; cursor: pointer; margin-top: 4px; }
  .btn-primary:disabled { opacity: 0.4; cursor: default; }

  .bm-list { display: flex; flex-direction: column; gap: 8px; }
  .bm-card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 10px 14px; border-left: 3px solid var(--primary); }
  .bm-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .bm-file { font-family: "SF Mono", Menlo, monospace; font-size: 12px; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
  .btn-delete { background: transparent; border: none; color: #555; cursor: pointer; font-size: 12px; padding: 2px 6px; border-radius: 4px; flex-shrink: 0; }
  .btn-delete:hover { background: rgba(239,68,68,0.15); color: #ef4444; }
  .bm-meta { display: flex; gap: 8px; margin: 6px 0 4px; flex-wrap: wrap; }
  .bm-tag { padding: 1px 8px; border-radius: 10px; background: rgba(59,130,246,0.1); color: var(--primary); font-size: 10px; font-weight: 600; }
  .bm-offset { font-family: monospace; font-size: 10px; color: var(--text-secondary); }
  .bm-note { font-size: 12px; color: var(--text-secondary); margin: 4px 0; line-height: 1.4; }
  .bm-time { font-size: 10px; color: var(--text-muted); }
  .spinner { display: inline-block; animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 60px 20px; text-align: center; }
  .empty-state .icon { font-size: 36px; opacity: 0.3; margin-bottom: 12px; }
  .empty-state p { font-size: 14px; color: var(--text-secondary); margin: 0; }
  .hint { font-size: 12px; color: var(--text-muted); margin-top: 6px; }

  .loading-state { display: flex; align-items: center; justify-content: center; padding: 60px; color: var(--text-secondary); gap: 8px; font-size: 14px; }
  .loading-state .spinner { animation: spin 1s linear infinite; }
</style>
