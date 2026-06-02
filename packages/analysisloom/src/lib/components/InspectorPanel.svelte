<script>
let { metadata, visible } = $props();

function entropyLabel(e) {
    if (e === null || e === undefined) return "—";
    if (e < 4.0) return "Low";
    if (e < 6.5) return "Medium";
    return "High (encrypted/compressed)";
}

function sizeStr(bytes) {
    if (!bytes) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let i = 0; let s = bytes;
    while (s >= 1024 && i < units.length - 1) { s /= 1024; i++; }
    return `${s.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}
</script>

{#if visible}
<aside class="inspector">
    <div class="head">
        <h3>📋 Inspector</h3>
    </div>

    {#if metadata}
        <section class="section">
            <h4>File Info</h4>
            <div class="grid">
                <span class="key">Size</span>
                <span class="val">{sizeStr(metadata.size)}</span>
                <span class="key">Modified</span>
                <span class="val mono">{metadata.modified}</span>
                <span class="key">Created</span>
                <span class="val mono">{metadata.created}</span>
                <span class="key">Permissions</span>
                <span class="val mono">{metadata.permissions}</span>
                <span class="key">Directory</span>
                <span class="val">{metadata.isDir ? "✅ Yes" : "❌ No"}</span>
            </div>
        </section>

        <section class="section">
            <h4>🔐 Integrity</h4>
            <div class="grid">
                <span class="key">MD5</span>
                <span class="val mono small">{metadata.md5 || "—"}</span>
                <span class="key">SHA-1</span>
                <span class="val mono small">{metadata.sha1 || "—"}</span>
                <span class="key">SHA-256</span>
                <span class="val mono small">{metadata.sha256 || "—"}</span>
            </div>
        </section>

        <section class="section">
            <h4>🔬 Analysis</h4>
            <div class="grid">
                <span class="key">Magic</span>
                <span class="val">{metadata.magicMatch || "Unknown"}</span>
                <span class="key">Entropy</span>
                <span class="val">{metadata.entropy ? metadata.entropy.toFixed(2) : "—"} <span class="hint">({entropyLabel(metadata.entropy)})</span></span>
            </div>
        </section>
    {:else}
        <div class="empty">
            <p>Select a file to inspect</p>
        </div>
    {/if}
</aside>
{/if}

<style>
.inspector { width: 280px; min-width: 280px; background: #0d0d0d; border-left: 1px solid var(--border); overflow-y: auto; display: flex; flex-direction: column; }
.head { padding: 12px 14px; border-bottom: 1px solid var(--border); }
.head h3 { margin: 0; font-size: 13px; color: #ccc; }
.section { padding: 10px 14px; border-bottom: 1px solid var(--border); }
.section h4 { margin: 0 0 8px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); }
.grid { display: grid; grid-template-columns: auto 1fr; gap: 3px 10px; font-size: 11px; }
.key { color: var(--text-secondary); }
.val { color: #d4d4d4; word-break: break-all; }
.mono { font-family: "SF Mono","Menlo","Cascadia Code",monospace; }
.small { font-size: 10px; }
.hint { color: var(--text-secondary); font-size: 10px; }
.empty { display: flex; align-items: center; justify-content: center; flex: 1; color: var(--text-secondary); font-size: 12px; padding: 20px; text-align: center; }
</style>
