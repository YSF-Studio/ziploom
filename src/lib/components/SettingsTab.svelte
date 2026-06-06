<script>
  import ThemeSelector from "./ThemeSelector.svelte";
  import { invoke } from "../tauri.js";
  import { getPrefs, savePrefs } from "../prefs.js";
  import { notify } from "../toast.js";

  let { onToast, onPrefsChange } = $props();
  let prefs = $state(getPrefs());
  let tools = $state(null);
  let toolsLoading = $state(false);

  function update(patch) {
    prefs = savePrefs(patch);
    onPrefsChange?.(prefs);
  }

  async function loadTools() {
    toolsLoading = true;
    try {
      tools = await invoke("check_tools");
    } catch (e) {
      notify(onToast, String(e), "error");
    } finally {
      toolsLoading = false;
    }
  }
</script>

<div class="page">
  <div class="panel">
    <h3>Appearance</h3>
    <p class="hint">Choose the application theme.</p>
    <ThemeSelector value={prefs.theme} onChange={(t) => update({ theme: t })} />
  </div>

  <div class="panel">
    <h3>Defaults</h3>
    <div class="row">
      <label for="def-fmt">Default compression format</label>
      <select id="def-fmt" value={prefs.defaultFormat} onchange={(e) => update({ defaultFormat: e.currentTarget.value })}>
        <option value="zip">ZIP</option>
        <option value="tar">TAR</option>
        <option value="tar.gz">TAR.GZ</option>
      </select>
    </div>
  </div>

  <details class="details advanced panel-flat">
    <summary>Advanced settings</summary>
    <div class="advanced-body">
      <div class="row">
        <label for="def-level">Default compression level</label>
        <input
          id="def-level"
          type="range"
          min="0"
          max="9"
          value={prefs.defaultLevel}
          oninput={(e) => update({ defaultLevel: Number(e.currentTarget.value) })}
        />
        <span class="level-val">{prefs.defaultLevel}</span>
      </div>

      <label class="check">
        <input
          type="checkbox"
          checked={prefs.confirmDestructive}
          onchange={(e) => update({ confirmDestructive: e.currentTarget.checked })}
        />
        Confirm before destructive actions
      </label>

      <div>
        <div class="row">
          <span class="subhead">Tool availability</span>
          <button class="btn btn-ghost" disabled={toolsLoading} onclick={loadTools}>
            {toolsLoading ? "Checking…" : tools ? "Refresh" : "Check tools"}
          </button>
        </div>
        {#if tools}
          <ul class="tool-list">
            {#each tools as t}
              <li class:ok={t.available} class:missing={!t.available}>
                <span class="dot"></span>
                {t.name}
                <span class="status">{t.available ? "available" : "not found"}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
  </details>
</div>

<style>
  .hint { margin: 0 0 12px; font-size: 13px; color: var(--muted); }
  .panel-flat { margin-top: 16px; padding: 14px; border: 1px solid var(--border); border-radius: var(--radius-lg); background: var(--surface); }
  .advanced-body { display: flex; flex-direction: column; gap: 12px; }
  .check { display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; }
  .level-val { font-size: 12px; color: var(--muted); min-width: 16px; }
  .subhead { font-size: 13px; font-weight: 600; }
  .tool-list { list-style: none; margin: 8px 0 0; padding: 0; font-size: 12px; }
  .tool-list li { display: flex; align-items: center; gap: 8px; padding: 4px 0; color: var(--muted); }
  .tool-list .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--err); }
  .tool-list li.ok .dot { background: var(--ok); }
  .tool-list .status { margin-left: auto; font-size: 11px; }
</style>
