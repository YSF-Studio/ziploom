<script>
import { invoke } from "@tauri-apps/api/core";
import CaseTab from "./lib/components/CaseTab.svelte";
import FileBrowserTab from "./lib/components/FileBrowserTab.svelte";
import CarvingTab from "./lib/components/CarvingTab.svelte";
import TimelineTab from "./lib/components/TimelineTab.svelte";
import SearchTab from "./lib/components/SearchTab.svelte";

let activeTab = $state(0);
let msg = $state("");
let busy = $state(false);
let activeCase = $state(null);

function timeoutPromise(promise, ms) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject("TIMEOUT"), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

const tabs = [
  { label: "📁 Cases" },
  { label: "🗂️ File Browser" },
  { label: "🔍 Carving" },
  { label: "📊 Timeline" },
  { label: "🔎 Search" },
];
</script>

<div class="app">
  <header class="header">
    <h1>🔬 AnalysisLoom <span class="ver">v0.1.0</span></h1>
    <span class="subtitle">Forensic Analysis Workstation</span>
  </header>

  <nav class="tabs">
    {#each tabs as tab, i}
      <button class="tab" class:active={activeTab === i} onclick={() => activeTab = i}>{tab.label}</button>
    {/each}
  </nav>

  <main class="content">
    {#if activeTab === 0}<CaseTab bind:activeCase bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 1}<FileBrowserTab bind:activeCase bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 2}<CarvingTab bind:activeCase bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 3}<TimelineTab bind:activeCase bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 4}<SearchTab bind:activeCase bind:busy bind:msg {timeoutPromise} />{/if}
  </main>

  {#if msg}
    <div class="toast" class:error={msg.includes("❌")} class:success={msg.includes("✅")}>
      {msg}
      <button class="close-toast" onclick={() => msg = ""}>✕</button>
    </div>
  {/if}
</div>

<style>
:global(body) { margin:0; font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif; background:#0a0a0a; color:#e0e0e0; --card:#141414; --border:#2a2a2a; --primary:#3b82f6; --success:#22c55e; --danger:#ef4444; --warn:#f59e0b; --text-secondary:#86868b; }
.app { max-width:1200px; margin:0 auto; padding:0 16px 40px; }
.header { padding:20px 0; border-bottom:1px solid var(--border); margin-bottom:12px; }
.header h1 { margin:0; font-size:22px; }
.ver { font-size:12px; color:var(--text-secondary); font-weight:normal; }
.subtitle { font-size:12px; color:var(--text-secondary); }
.tabs { display:flex; gap:2px; flex-wrap:wrap; margin-bottom:16px; }
.tab { padding:8px 14px; border:1px solid var(--border); border-radius:8px 8px 0 0; background:var(--card); color:var(--text-secondary); cursor:pointer; font-size:13px; transition:all 0.15s; }
.tab:hover { color:#e0e0e0; }
.tab.active { background:#1a1a2e; color:var(--primary); border-bottom-color:transparent; font-weight:600; }
.content { background:var(--card); border:1px solid var(--border); border-radius:0 10px 10px 10px; padding:20px; min-height:400px; }
.toast { position:fixed; bottom:20px; right:20px; padding:12px 18px; border-radius:10px; background:#1a2e1a; border:1px solid var(--success); font-size:13px; max-width:400px; z-index:1000; }
.toast.error { background:#2e1a1a; border-color:var(--danger); }
.close-toast { background:none; border:none; color:inherit; cursor:pointer; margin-left:10px; }
</style>