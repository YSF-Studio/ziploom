<script>
let { busy, activeCase, activeFile } = $props();

let time = $state(new Date().toLocaleTimeString());
$effect(() => {
    const id = setInterval(() => time = new Date().toLocaleTimeString(), 10000);
    return () => clearInterval(id);
});
</script>

<div class="statusbar">
    <div class="left">
        {#if busy}
            <span class="spinner">⏳</span>
            <span>Processing...</span>
        {:else}
            <span class="dot" class:active={!!activeCase}></span>
            <span>{activeCase ? `Case: ${activeCase}` : "No case selected"}</span>
        {/if}
    </div>
    <div class="center">
        {#if activeFile}
            <span title={activeFile}>{activeFile.split("/").pop()}</span>
        {/if}
    </div>
    <div class="right">
        <span class="offline-badge">🔒 Offline</span>
        <span class="time">{time}</span>
    </div>
</div>

<style>
.statusbar { display: flex; align-items: center; justify-content: space-between; padding: 4px 12px; background: #0d0d0d; border-top: 1px solid var(--border); font-size: 11px; color: var(--text-secondary); height: 28px; user-select: none; }
.left, .center, .right { display: flex; align-items: center; gap: 6px; }
.center { flex: 1; justify-content: center; overflow: hidden; }
.center span { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 300px; }
.dot { width: 8px; height: 8px; border-radius: 50%; background: #555; }
.dot.active { background: var(--success); box-shadow: 0 0 4px var(--success); }
.spinner { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
.offline-badge { padding: 1px 8px; background: rgba(34,197,94,0.15); color: var(--success); border-radius: 10px; font-size: 10px; font-weight: 600; }
.time { min-width: 70px; text-align: right; }
</style>
