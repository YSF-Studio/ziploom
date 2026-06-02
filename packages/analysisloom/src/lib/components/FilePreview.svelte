<script>
import { invoke } from "@tauri-apps/api/core";

let { filePath, busy, msg, timeoutPromise, onPreview } = $props();

let preview = $state(null);
let loading = $state(false);

export async function loadPreview(path) {
    if (!path) return;
    loading = true;
    try {
        const result = await timeoutPromise(
            invoke("preview_file", { path }),
            30000
        );
        preview = result;
        if (onPreview) onPreview(result);
    } catch(e) {
        preview = { kind: "Unsupported", preview: { type: "Unsupported", msg: String(e) } };
    }
    loading = false;
}

export function clear() {
    preview = null;
}

$effect(() => {
    if (filePath) loadPreview(filePath);
});

function sizeStr(bytes) {
    if (!bytes) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let i = 0; let s = bytes;
    while (s >= 1024 && i < units.length - 1) { s /= 1024; i++; }
    return `${s.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}
</script>

{#if loading}
    <div class="loading"><span class="spinner">⏳</span> Loading preview...</div>
{:else if preview}
    <div class="preview">
        {#if preview.preview?.Text}
            <pre class="text-view">{preview.preview.Text}</pre>
        {:else if preview.preview?.Image}
            <div class="image-view">
                <img src="data:image/png;base64,{preview.preview.Image.data_base64}" alt="preview"
                     style="max-width:100%;max-height:60vh;" />
                <span class="dim">{preview.preview.Image.width} × {preview.preview.Image.height}px</span>
            </div>
        {:else if preview.preview?.HexDump}
            <pre class="hex-view">{preview.preview.HexDump}</pre>
        {:else if preview.preview?.ArchiveList}
            <div class="archive-view">
                <h4>📦 Archive Contents ({preview.preview.ArchiveList.length} items)</h4>
                <div class="arc-list">
                    {#each preview.preview.ArchiveList as entry}
                        <div class="arc-item">{entry}</div>
                    {/each}
                </div>
            </div>
        {:else if preview.preview?.Unsupported}
            <div class="unsupported">
                <p>⚠️ {preview.preview.Unsupported}</p>
            </div>
        {/if}

        <div class="file-meta">
            <span class="label">Size:</span> {sizeStr(preview.size)}
            <span class="sep">|</span>
            <span class="label">Type:</span> {preview.kind}
            <span class="sep">|</span>
            <span class="label">MIME:</span> {preview.mime_type}
            <span class="sep">|</span>
            <span class="label">Ext:</span> {preview.extension}
        </div>
    </div>
{:else}
    <div class="empty">
        <p>Select a file to preview</p>
    </div>
{/if}

<style>
.loading { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-secondary); gap: 8px; font-size: 14px; }
.spinner { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.preview { display: flex; flex-direction: column; height: 100%; }
.text-view, .hex-view { flex: 1; overflow: auto; background: #0d0d0d; border: 1px solid var(--border); border-radius: 6px; padding: 12px; font-family: "SF Mono","Menlo","Cascadia Code",monospace; font-size: 12px; line-height: 1.5; color: #d4d4d4; white-space: pre-wrap; word-break: break-all; }
.hex-view { font-size: 11px; }
.image-view { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 16px; }
.image-view img { border-radius: 6px; border: 1px solid var(--border); }
.dim { font-size: 11px; color: var(--text-secondary); }
.archive-view { flex: 1; overflow: auto; }
.archive-view h4 { margin: 0 0 10px; font-size: 13px; }
.arc-list { display: flex; flex-direction: column; gap: 2px; }
.arc-item { padding: 4px 8px; font-family: "SF Mono","Menlo",monospace; font-size: 11px; color: var(--text-secondary); border-bottom: 1px solid var(--border); }
.unsupported { display: flex; align-items: center; justify-content: center; height: 150px; color: var(--warn); }
.file-meta { padding: 8px 12px; font-size: 11px; color: var(--text-secondary); background: #0d0d0d; border: 1px solid var(--border); border-radius: 6px; margin-top: 8px; }
.sep { margin: 0 6px; opacity: 0.3; }
.label { color: #888; }
.empty { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-secondary); font-size: 14px; }
</style>
