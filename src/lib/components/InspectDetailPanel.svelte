<script>
  import { formatSize } from "../format.js";
  import { analyzeEntry, threatLevel, threatLabel, copyText, entropyPercent } from "../inspect.js";

  let {
    entry = null,
    preview = null,
    previewLoading = false,
    archiveHashes = null,
    threats = [],
    anomalies = [],
    scanDone = false,
    threatByPath = new Map(),
    findingsTab = "threats",
    onFindingsTab,
    onJumpTo,
    onPreview,
  } = $props();

  function shortPath(p) {
    const parts = p.split("/");
    return parts[parts.length - 1] || p;
  }

  function imageSrc() {
    if (!preview?.image_base64 || !preview?.mime_type) return "";
    return `data:${preview.mime_type};base64,${preview.image_base64}`;
  }
</script>

<aside class="detail-panel">
  <div class="detail-section">
    <h4>Findings</h4>
    <div class="findings-tabs">
      <button class:active={findingsTab === "threats"} onclick={() => onFindingsTab?.("threats")}>
        Threats ({threats.length})
      </button>
      <button class:active={findingsTab === "anomalies"} onclick={() => onFindingsTab?.("anomalies")}>
        Anomalies ({anomalies.length})
      </button>
    </div>
    <ul class="findings-list">
      {#if findingsTab === "threats"}
        {#each threats.slice(0, 20) as t}
          <li>
            <button class="finding-link" onclick={() => onJumpTo?.(t.file)} title={t.detail}>
              {shortPath(t.file)}: {t.threat}
            </button>
          </li>
        {:else}
          <li class="muted">No threats detected</li>
        {/each}
      {:else}
        {#each anomalies.slice(0, 20) as a}
          <li>
            <button class="finding-link" onclick={() => onJumpTo?.(a.file)}>
              {shortPath(a.file)}: {a.issue}
            </button>
          </li>
        {:else}
          <li class="muted">No anomalies detected</li>
        {/each}
      {/if}
    </ul>
  </div>

  {#if archiveHashes}
    <div class="detail-section">
      <h4>Archive checksums</h4>
      {#if archiveHashes.md5}
        <button class="hash-row" onclick={() => copyText(archiveHashes.md5)} title="Click to copy">
          MD5: <code>{archiveHashes.md5}</code>
        </button>
      {/if}
      {#if archiveHashes.sha1}
        <button class="hash-row" onclick={() => copyText(archiveHashes.sha1)} title="Click to copy">
          SHA-1: <code>{archiveHashes.sha1}</code>
        </button>
      {/if}
      {#if archiveHashes.sha256}
        <button class="hash-row" onclick={() => copyText(archiveHashes.sha256)} title="Click to copy">
          SHA-256: <code>{archiveHashes.sha256}</code>
        </button>
      {/if}
    </div>
  {/if}

  <div class="detail-section">
    <h4>File detail</h4>
    {#if entry && !entry._folder && !entry.isDir}
      {@const level = threatLevel(entry, threatByPath, scanDone)}
      {@const flags = analyzeEntry(entry)}
      <dl class="meta-dl">
        <dt>Path</dt><dd title={entry.path}>{entry.path}</dd>
        <dt>Size</dt><dd>{formatSize(entry.size)}</dd>
        <dt>Threat</dt><dd>{threatLabel(level)}</dd>
        {#if entry.entropy != null}
          <dt>Entropy</dt>
          <dd>
            {Number(entry.entropy).toFixed(2)}
            <div class="ent-bar-lg" style="width: {entropyPercent(entry.entropy)}%"></div>
          </dd>
        {/if}
        {#if entry.detectedType}<dt>Magic</dt><dd>{entry.detectedType}</dd>{/if}
        {#if flags.length}<dt>Flags</dt><dd>{flags.join(", ")}</dd>{/if}
      </dl>
      <button class="btn-preview" disabled={previewLoading} onclick={() => onPreview?.(entry)}>
        {previewLoading ? "Loading preview…" : "Preview contents"}
      </button>
    {:else}
      <p class="muted">Select a file row to see details and preview.</p>
    {/if}
  </div>

  {#if preview}
    <div class="detail-section preview-section" class:unsafe={!preview.safe}>
      <h4>Preview</h4>
      {#if preview.warning}
        <p class="preview-warn">⚠ {preview.warning}</p>
      {/if}
      {#if preview.truncated}
        <p class="preview-note">Showing first {formatSize(preview.size)} (file may be larger).</p>
      {/if}
      {#if preview.preview_type === "image" && preview.image_base64}
        <img class="preview-img" src={imageSrc()} alt="Preview of {preview.path}" />
      {:else if preview.preview_type === "text" && preview.text}
        <pre class="preview-pre">{preview.text}</pre>
      {:else if preview.preview_type === "hex" && preview.hex}
        <pre class="preview-pre mono">{preview.hex}</pre>
      {:else}
        <p class="muted">Preview not available for this file type.</p>
      {/if}
      <p class="preview-safety">
        {#if preview.safe}
          Read-only preview — file is not extracted or executed.
        {:else}
          Potentially dangerous file — hex/text only, never executed.
        {/if}
      </p>
    </div>
  {/if}
</aside>

<style>
  .detail-panel {
    width: 300px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow-y: auto;
    min-height: 0;
    padding-left: 2px;
  }
  .detail-section {
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  h4 {
    margin: 0 0 8px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--muted);
  }
  .findings-tabs { display: flex; gap: 4px; margin-bottom: 8px; }
  .findings-tabs button {
    flex: 1; padding: 5px 8px; border-radius: 6px; border: 1px solid var(--border);
    background: var(--surface-soft); font-size: 11px; cursor: pointer; color: var(--muted);
  }
  .findings-tabs button.active { background: var(--accent); border-color: var(--accent); color: #fff; }
  .findings-list { margin: 0; padding-left: 0; list-style: none; font-size: 11px; }
  .findings-list li { margin-bottom: 6px; }
  .finding-link {
    border: none; background: none; padding: 0; text-align: left;
    color: var(--text); cursor: pointer; font-size: 11px; line-height: 1.35;
  }
  .finding-link:hover { color: var(--accent); text-decoration: underline; }
  .muted { color: var(--muted); font-size: 12px; margin: 0; }
  .hash-row {
    display: block; width: 100%; text-align: left; border: none; background: none;
    padding: 4px 0; cursor: pointer; font-size: 10px; color: var(--text);
  }
  .hash-row code { font-family: var(--mono); word-break: break-all; }
  .hash-row:hover { color: var(--accent); }
  .meta-dl { margin: 0 0 10px; font-size: 11px; }
  .meta-dl dt { color: var(--muted); margin-top: 6px; }
  .meta-dl dd { margin: 2px 0 0; word-break: break-all; }
  .ent-bar-lg {
    height: 4px; border-radius: 2px; margin-top: 4px;
    background: linear-gradient(90deg, var(--teal), var(--accent));
  }
  .btn-preview {
    width: 100%; padding: 8px; border-radius: var(--radius);
    border: 1px solid var(--accent); background: var(--accent-soft);
    color: var(--accent); font-size: 12px; font-weight: 600; cursor: pointer;
  }
  .btn-preview:disabled { opacity: 0.5; cursor: not-allowed; }
  .preview-section.unsafe { border-color: var(--warn); }
  .preview-warn {
    margin: 0 0 8px; padding: 8px; border-radius: 6px;
    background: var(--warn-bg); color: var(--warn); font-size: 11px;
  }
  .preview-note { margin: 0 0 8px; font-size: 11px; color: var(--muted); }
  .preview-pre {
    margin: 0; max-height: 220px; overflow: auto;
    padding: 8px; border-radius: 6px; background: var(--surface-soft);
    font-size: 10px; line-height: 1.4; white-space: pre-wrap; word-break: break-all;
  }
  .preview-pre.mono { font-family: var(--mono); white-space: pre; }
  .preview-img { max-width: 100%; max-height: 180px; border-radius: 6px; }
  .preview-safety { margin: 8px 0 0; font-size: 10px; color: var(--muted); }
</style>
