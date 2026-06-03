<script>
import { invoke } from "@tauri-apps/api/core";

let { setBusy, setMsg, timeoutPromise } = $props();
let loading = $state(false);
let snapshotResult = $state(null);
let compareResult = $state(null);

async function takeSnapshot() {
    loading = true; setBusy(true);
    try {
        const result = await timeoutPromise(invoke("take_snapshot"), 60000);
        snapshotResult = result;
        setMsg("✅ Snapshot captured successfully");
    } catch(e) {
        setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
    }
    loading = false; setBusy(false);
}

async function compareSnapshot() {
    if (!snapshotResult) return;
    loading = true; setBusy(true);
    try {
        const diff = await timeoutPromise(invoke("compare_snapshot", {
            previousId: snapshotResult.id
        }), 60000);
        compareResult = diff;
        setMsg(`✅ Diff complete — ${diff.changes || "no"} changes detected`);
    } catch(e) {
        setMsg(`❌ ${typeof e === "string" ? e : String(e)}`);
    }
    loading = false; setBusy(false);
}

function riskColor(level) {
    if (!level) return "#555";
    if (level === "LOW") return "#22c55e";
    if (level === "MEDIUM") return "#f59e0b";
    return "#ef4444";
}
</script>

<div class="snapshot-tab">
    <h3>📸 System Snapshot</h3>
    <p class="desc">Capture point-in-time system state for forensic comparison and integrity verification.</p>

    <div class="card">
        <h4>Take Snapshot</h4>
        <p style="color:var(--text-secondary);font-size:12px;margin:0 0 12px">
            Captures filesystem state, running processes, network connections, and system information.
        </p>
        <button class="btn-primary" onclick={takeSnapshot} disabled={loading || busy}>
            {loading ? "⏳ Capturing..." : "📸 Capture Snapshot"}
        </button>
    </div>

    {#if snapshotResult}
        <div class="card result-card">
            <h4>Last Snapshot</h4>
            <div class="info-grid">
                <span class="key">ID</span>
                <span class="val mono">{snapshotResult.id}</span>
                <span class="key">Timestamp</span>
                <span class="val">{snapshotResult.timestamp || "—"}</span>
                <span class="key">Files</span>
                <span class="val">{snapshotResult.file_count || 0}</span>
                <span class="key">Processes</span>
                <span class="val">{snapshotResult.process_count || 0}</span>
                <span class="key">Network</span>
                <span class="val">{snapshotResult.network_count || 0} connections</span>
            </div>
            <button class="btn" onclick={compareSnapshot} disabled={loading || busy} style="margin-top:12px">
                🔄 Compare with Current State
            </button>
        </div>
    {/if}

    {#if compareResult}
        <div class="card diff-card">
            <h4>Diff Results</h4>
            <div class="info-grid">
                <span class="key">Risk Level</span>
                <span class="val" style="color:{riskColor(compareResult.risk_level)};font-weight:600">
                    {compareResult.risk_level || "NONE"}
                </span>
                <span class="key">New Files</span>
                <span class="val">{compareResult.new_files || 0}</span>
                <span class="key">Deleted</span>
                <span class="val">{compareResult.deleted_files || 0}</span>
                <span class="key">Modified</span>
                <span class="val">{compareResult.modified_files || 0}</span>
            </div>
            {#if compareResult.report}
                <pre class="diff-report">{compareResult.report}</pre>
            {/if}
        </div>
    {/if}

    {#if !snapshotResult && !loading}
        <div class="empty-state">
            <span class="icon">📸</span>
            <p>No snapshots taken yet</p>
            <span style="font-size:11px;color:var(--text-muted)">Capture a snapshot to begin system monitoring</span>
        </div>
    {/if}
</div>

<style>
.snapshot-tab { max-width: 700px; }
.snapshot-tab h3 { margin: 0 0 4px; font-size: 16px; }
.desc { color: var(--text-secondary); font-size: 13px; margin: 0 0 20px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 16px 20px; margin-bottom: 14px; }
.card h4 { margin: 0 0 8px; font-size: 13px; color: #ccc; }
.btn { padding: 6px 14px; border: 1px solid var(--border); background: var(--card); color: var(--text-secondary); border-radius: 6px; cursor: pointer; font-size: 12px; }
.btn:hover { background: var(--card-hover); color: var(--text); }
.btn-primary { padding: 8px 18px; background: var(--primary); color: white; border: none; border-radius: 8px; font-weight: 600; font-size: 13px; cursor: pointer; }
.btn-primary:disabled { opacity: 0.4; cursor: default; }
.result-card { border-left: 3px solid var(--primary); }
.diff-card { border-left: 3px solid var(--warn); }
.info-grid { display: grid; grid-template-columns: auto 1fr; gap: 4px 12px; font-size: 12px; }
.info-grid .key { color: var(--text-secondary); }
.info-grid .val { color: #d4d4d4; }
.mono { font-family: var(--mono); font-size: 11px; }
.diff-report { margin-top: 12px; padding: 10px; background: #0d0d0d; border: 1px solid var(--border); border-radius: 6px; font-size: 11px; color: #d4d4d4; overflow-x: auto; max-height: 300px; overflow-y: auto; }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 160px; color: var(--text-muted); gap: 8px; }
.empty-state .icon { font-size: 36px; opacity: 0.3; }
.empty-state p { margin: 0; font-size: 14px; }
</style>
