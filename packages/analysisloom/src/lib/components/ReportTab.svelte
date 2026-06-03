<script>
  import { invoke } from "@tauri-apps/api/core";

  let { activeCase, busy, msg, timeoutPromise } = $props();

  let format = $state("html");
  let generating = $state(false);
  let reportPath = $state("");
  let auditLog = $state([]);

  async function generateReport() {
    if (!activeCase?.id) return;
    generating = true;
    reportPath = "";
    try {
      const path = await invoke("generate_case_report", {
        caseId: activeCase.id,
        format,
      });
      reportPath = path;
      // Log the action
      await invoke("log_action", {
        caseId: activeCase.id,
        action: "GENERATE_REPORT",
        detail: format.toUpperCase() + " report generated",
      });
    } catch (e) {
      const err = typeof e === 'string' ? e : String(e);
      msg.set(`❌ ${err}`);
    }
    generating = false;
    busy.set(false);
  }

  async function loadAuditLog() {
    if (!activeCase?.id) return;
    try {
      auditLog = await invoke("get_audit_log", { caseId: activeCase.id });
    } catch (e) {}
  }

  // Load audit log when case changes
  $effect(() => {
    if (activeCase?.id) loadAuditLog();
  });

  function severityColor(s) {
    if (s === "critical") return "var(--danger)";
    if (s === "warning") return "var(--warn)";
    return "var(--text-secondary)";
  }
</script>

<div class="report-container">
  <h3>📄 Forensic Report</h3>
  <p class="subtitle">
    Generate a comprehensive forensic report (PDF or HTML) from case data — timeline, evidence, findings, and audit trail.
  </p>

  {#if !activeCase?.id}
    <div class="empty-state">
      <span style="font-size:32px">📂</span>
      <p>Open a case first from the Case Manager</p>
    </div>
  {:else}
    <!-- Format selection -->
    <div class="card">
      <h4>Report Configuration</h4>
      <div class="row">
        <label>Case: <strong>{activeCase.name}</strong></label>
      </div>
      <div class="row">
        <label>Format:</label>
        <div class="format-pills">
          <button class="pill" class:active={format === 'html'} onclick={() => format = 'html'}>
            🌐 HTML Report
          </button>
          <button class="pill" class:active={format === 'pdf'} onclick={() => format = 'pdf'}>
            📕 PDF Report
          </button>
        </div>
      </div>

      <button class="btn-generate" onclick={generateReport} disabled={generating}>
        {generating ? '🔄 Generating...' : '📄 Generate Report'}
      </button>

      {#if reportPath}
        <div class="report-link">
          <span class="icon">✅</span>
          <div>
            <strong>Report saved:</strong><br />
            <code>{reportPath}</code>
          </div>
        </div>
      {/if}
    </div>

    <!-- Audit Trail -->
    <div class="card" style="margin-top:16px">
      <h4>📋 Audit Trail (Last 50 Actions)</h4>
      {#if auditLog.length > 0}
        <div class="audit-table">
          {#each auditLog as entry}
            <div class="audit-row">
              <span class="audit-time">{entry.timestamp}</span>
              <span class="audit-action">{entry.action}</span>
              <span class="audit-detail">{entry.detail}</span>
            </div>
          {/each}
        </div>
      {:else}
        <p class="muted">No audit log entries yet. Actions will be recorded automatically.</p>
      {/if}
    </div>

    <!-- Report Preview -->
    <div class="card" style="margin-top:16px">
      <h4>📊 Report Contents Preview</h4>
      <p class="muted">The report includes all sections below from your active case:</p>
      <ul class="preview-list">
        <li><strong>Case Information</strong> — Name, operator, status, creation date</li>
        <li><strong>Timeline Events</strong> — Chronological event log (last 100)</li>
        <li><strong>Evidence Items</strong> — All acquired items with hashes</li>
        <li><strong>Findings</strong> — Tagged findings with severity</li>
        <li><strong>Audit Trail</strong> — Complete action log with timestamps</li>
      </ul>
    </div>
  {/if}
</div>

<style>
  .report-container h3 { margin: 0 0 4px; font-size: 18px; color: var(--text); }
  .subtitle { color: var(--text-secondary); font-size: 13px; margin: 0 0 20px; }

  .card {
    background: var(--card); border: 1px solid var(--border);
    border-radius: var(--radius-lg); padding: 16px;
  }
  .card h4 { margin: 0 0 12px; font-size: 14px; color: var(--text); }

  .row { margin-bottom: 12px; font-size: 13px; color: var(--text-secondary); }

  .format-pills { display: flex; gap: 8px; margin-top: 4px; }
  .pill {
    padding: 6px 16px; border: 1px solid var(--border); border-radius: 20px;
    background: transparent; color: var(--text-secondary); font-size: 12px;
    cursor: pointer; transition: all 0.15s;
  }
  .pill:hover { border-color: var(--primary); color: var(--text); }
  .pill.active {
    background: var(--primary); border-color: var(--primary); color: #fff; font-weight: 600;
  }

  .btn-generate {
    padding: 10px 24px; background: var(--primary); color: #fff;
    border: none; border-radius: 8px; font-size: 13px; font-weight: 600;
    cursor: pointer; margin-top: 4px; transition: filter 0.15s;
  }
  .btn-generate:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-generate:disabled { opacity: 0.4; cursor: not-allowed; }

  .report-link {
    display: flex; align-items: flex-start; gap: 10px;
    margin-top: 12px; padding: 10px; border-radius: 8px;
    background: rgba(34,197,94,0.08); border: 1px solid var(--success);
    font-size: 13px;
  }
  .report-link .icon { font-size: 18px; flex-shrink: 0; }
  .report-link code {
    font-family: var(--mono); font-size: 11px; color: var(--text-secondary);
    word-break: break-all;
  }

  /* Audit table */
  .audit-table { max-height: 240px; overflow-y: auto; }
  .audit-row {
    display: grid; grid-template-columns: 140px 120px 1fr; gap: 8px;
    padding: 6px 4px; border-bottom: 1px solid rgba(255,255,255,0.03);
    font-size: 11px;
  }
  .audit-time { color: var(--text-muted); font-family: var(--mono); }
  .audit-action { color: var(--primary); font-weight: 600; font-family: var(--mono); }
  .audit-detail { color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .preview-list { margin: 8px 0 0; padding-left: 20px; }
  .preview-list li { font-size: 12px; color: var(--text-secondary); margin-bottom: 4px; }

  .empty-state {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    padding: 60px 20px; text-align: center;
  }
  .empty-state p { color: var(--text-muted); font-size: 14px; margin-top: 12px; }

  .muted { color: var(--text-muted); font-size: 12px; }
</style>
