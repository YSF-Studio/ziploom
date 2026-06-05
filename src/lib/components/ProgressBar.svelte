<script>
  import { formatSize } from "../format.js";

  /** @type {{ progress: import('../progressPoll.js').ProgressState | null }} */
  let { progress = null } = $props();

  const show = $derived(
    progress &&
      (progress.status ||
        progress.percent > 0 ||
        progress.bytes_processed > 0 ||
        progress.is_done)
  );

  const pct = $derived(Math.min(100, Math.max(0, progress?.percent ?? 0)));
</script>

{#if show}
  <div
    class="progress-bar-wrap"
    role="progressbar"
    aria-valuenow={pct}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-label={progress.status || "Working"}
  >
    <div class="progress-track">
      <div class="progress-fill" class:indeterminate={pct < 1 && !progress.is_done} style="width: {pct}%"></div>
    </div>
    <div class="progress-meta">
      <span class="progress-status">{progress.status || "Working…"}</span>
      {#if progress.total_bytes > 0}
        <span class="progress-bytes">
          {formatSize(progress.bytes_processed)} / {formatSize(progress.total_bytes)}
        </span>
      {/if}
      <span class="progress-pct">{Math.round(pct)}%</span>
    </div>
  </div>
{/if}

<style>
  .progress-bar-wrap {
    margin: 0 0 12px;
    padding: 10px 12px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .progress-track {
    height: 8px;
    border-radius: 4px;
    background: var(--surface-soft);
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    border-radius: 4px;
    background: linear-gradient(90deg, var(--accent), var(--teal));
    transition: width 0.2s ease;
  }
  .progress-fill.indeterminate {
    width: 40% !important;
    animation: progress-indeterminate 1.2s ease-in-out infinite;
  }
  @keyframes progress-indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(350%); }
  }
  .progress-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 8px;
    font-size: 11px;
    color: var(--muted);
  }
  .progress-status {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
  }
  .progress-pct {
    font-weight: 700;
    color: var(--accent);
    font-variant-numeric: tabular-nums;
  }
</style>
