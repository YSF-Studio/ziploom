<script>
  import { formatSize } from "../format.js";
  import { analyzeEntry, threatLevel, threatLabel, copyText, entropyPercent, isMagicMismatch, isSuspiciousTimestamp } from "../inspect.js";

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

  let panelTab = $state("findings");

  const summary = $derived.by(() => {
    const all = [...threats, ...anomalies.map((a) => ({
      file: a.file,
      threat: a.issue,
      category: "anomaly",
      severity: a.severity ?? "low",
      detail: a.issue,
    }))];
    const byCategory = new Map();
    const bySeverity = new Map();
    for (const t of all) {
      byCategory.set(t.category, (byCategory.get(t.category) ?? 0) + 1);
      bySeverity.set(t.severity, (bySeverity.get(t.severity) ?? 0) + 1);
    }
    return { total: all.length, byCategory, bySeverity };
  });

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
  <div class="panel-nav">
    <button class:active={panelTab === "summary"} onclick={() => (panelTab = "summary")}>Summary</button>
    <button class:active={panelTab === "findings"} onclick={() => (panelTab = "findings")}>Findings</button>
    <button class:active={panelTab === "checksums"} onclick={() => (panelTab = "checksums")}>Checksums</button>
    <button class:active={panelTab === "details"} onclick={() => (panelTab = "details")}>File detail</button>
    <button class:active={panelTab === "preview"} onclick={() => (panelTab = "preview")}>Preview</button>
  </div>

  {#if panelTab === "summary"}
    <div class="detail-section">
      <h4>Summary</h4>
      <div class="summary-grid">
        <div class="summary-card">
          <span class="k">Total findings</span>
          <strong>{summary.total}</strong>
        </div>
        <div class="summary-card">
          <span class="k">Threats</span>
          <strong>{threats.length}</strong>
        </div>
        <div class="summary-card">
          <span class="k">Anomalies</span>
          <strong>{anomalies.length}</strong>
        </div>
        <div class="summary-card">
          <span class="k">Scan state</span>
          <strong>{scanDone ? "Done" : "Pending"}</strong>
        </div>
      </div>
      <div class="summary-list">
        <div><span>High</span><strong>{summary.bySeverity.get("high") ?? 0}</strong></div>
        <div><span>Critical</span><strong>{summary.bySeverity.get("critical") ?? 0}</strong></div>
        <div><span>Medium</span><strong>{summary.bySeverity.get("medium") ?? 0}</strong></div>
        <div><span>Low</span><strong>{summary.bySeverity.get("low") ?? 0}</strong></div>
      </div>
      <div class="summary-tags">
        {#each [...summary.byCategory.entries()].sort((a,b)=>b[1]-a[1]).slice(0, 8) as [cat, count]}
          <span>{cat}: {count}</span>
        {/each}
      </div>
    </div>
  {:else if panelTab === "findings"}
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
  {:else if panelTab === "checksums"}
    <div class="detail-section">
      <h4>Archive checksums</h4>
      {#if archiveHashes}
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
      {:else}
        <p class="muted">Run “Hash Archive” to generate container checksums.</p>
      {/if}
    </div>
  {:else if panelTab === "details"}
    <div class="detail-section">
      <h4>File detail</h4>
      {#if entry && !entry._folder && !entry.isDir}
        {@const level = threatLevel(entry, threatByPath, scanDone)}
        {@const flags = analyzeEntry(entry)}
        {@const mismatch = isMagicMismatch(entry)}
        <dl class="meta-dl">
          <dt>Path</dt><dd title={entry.path}>{entry.path}</dd>
          <dt>Size</dt><dd>{formatSize(entry.size)}</dd>
          {#if entry.timestamp || entry.modified}
            <dt>Timestamp</dt><dd class:warn={isSuspiciousTimestamp(entry.timestamp ?? entry.modified)} title={entry.timestamp ?? entry.modified}>{entry.timestamp ?? entry.modified}</dd>
          {/if}
          <dt>Threat</dt><dd>{threatLabel(level)}</dd>
          {#if entry.detectedType || entry.expectedType || entry.magicMatch != null}
            <dt>Magic</dt>
            <dd class:warn={mismatch}>
              {entry.detectedType ?? "Unknown"}
              {#if entry.expectedType}
                <span class="meta-sub">vs .{entry.expectedType}</span>
              {/if}
              {#if entry.magicMatch === false}
                <span class="meta-badge warn">Mismatch</span>
              {:else if entry.magicMatch === true}
                <span class="meta-badge ok">Match</span>
              {/if}
            </dd>
          {/if}
          {#if entry.entropy != null}
            <dt>Entropy</dt>
            <dd>
              {Number(entry.entropy).toFixed(2)}
              <div class="ent-bar-lg" style="width: {entropyPercent(entry.entropy)}%"></div>
            </dd>
          {/if}
          {#if flags.length}<dt>Flags</dt><dd>{flags.join(", ")}</dd>{/if}
        </dl>
        <button
          class="btn-preview"
          disabled={previewLoading}
          onclick={async () => {
            panelTab = "preview";
            await onPreview?.(entry);
          }}
        >
          {previewLoading ? "Loading preview…" : "Preview contents"}
        </button>
      {:else}
        <p class="muted">Select a file row to see details and preview.</p>
      {/if}
    </div>
  {:else}
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
    {:else}
      <div class="detail-section">
        <h4>Preview</h4>
        <p class="muted">Select a file and click “Preview contents”.</p>
      </div>
    {/if}
  {/if}
</aside>

<style>
  .detail-panel {
    width: 380px;
    min-width: 320px;
    max-width: 580px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    padding-left: 2px;
    resize: horizontal;
  }
  .panel-nav {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 6px;
    position: sticky;
    top: 0;
    z-index: 2;
    padding: 0 0 2px;
    background: linear-gradient(180deg, var(--bg) 0%, color-mix(in srgb, var(--bg) 88%, transparent) 100%);
    backdrop-filter: blur(8px);
  }
  .panel-nav button {
    padding: 7px 8px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }
  .panel-nav button.active {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }
  .summary-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
    margin-bottom: 10px;
  }
  .summary-card {
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface-soft);
  }
  .summary-card .k { display: block; font-size: 10px; color: var(--muted); margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.04em; }
  .summary-card strong { font-size: 15px; }
  .summary-list {
    display: grid;
    gap: 6px;
    margin-bottom: 10px;
  }
  .summary-list > div,
  .summary-tags span {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    font-size: 11px;
  }
  .summary-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
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
  .preview-img { max-width: 100%; max-height: 260px; border-radius: 6px; }
  .preview-safety { margin: 8px 0 0; font-size: 10px; color: var(--muted); }
  .warn { color: var(--warn); font-weight: 700; }
  .meta-sub { display: block; margin-top: 2px; font-size: 10px; color: var(--muted); }
  .meta-badge {
    display: inline-block;
    margin-top: 4px;
    padding: 1px 6px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 700;
  }
  .meta-badge.warn { background: var(--warn-bg); color: var(--warn); }
  .meta-badge.ok { background: var(--ok-soft); color: var(--ok); }
</style>
