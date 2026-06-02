<script>
import { invoke } from "@tauri-apps/api/core";
import DiskTab from "./lib/components/DiskTab.svelte";
import RamTab from "./lib/components/RamTab.svelte";
import WriteBlockerTab from "./lib/components/WriteBlockerTab.svelte";
import MobileTab from "./lib/components/MobileTab.svelte";
import CloudTab from "./lib/components/CloudTab.svelte";
import NetworkTab from "./lib/components/NetworkTab.svelte";
import EncryptionTab from "./lib/components/EncryptionTab.svelte";
import CocTab from "./lib/components/CocTab.svelte";

let activeTab = $state(0);
let msg = $state("");
let busy = $state(false);

const tabs = [
  { label: "📀 Disk", icon: "💿" },
  { label: "🧠 RAM", icon: "🧠" },
  { label: "🛡️ Write Blocker", icon: "🛡️" },
  { label: "📱 Mobile", icon: "📱" },
  { label: "☁️ Cloud", icon: "☁️" },
  { label: "🌐 Network", icon: "🌐" },
  { label: "🔐 Encryption", icon: "🔐" },
  { label: "📋 Chain of Custody", icon: "📋" },
];

// Share state between tabs
let diskState = $state({});
let ramState = $state({});
let encryptionState = $state({});
let cocState = $state({});

// Timeout wrapper — proven pattern from ZipLoom
function timeoutPromise(promise, ms) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject("TIMEOUT"), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}
</script>

<div class="app">
  <header class="header">
    <h1>🟢 CollectionLoom <span class="ver">v0.1.0</span></h1>
    <span class="subtitle">Portable Forensic Acquisition Toolkit — ISO 27037</span>
  </header>

  <nav class="tabs">
    {#each tabs as tab, i}
      <button class="tab" class:active={activeTab === i} onclick={() => activeTab = i}>
        {tab.label}
      </button>
    {/each}
  </nav>

  <main class="content">
    {#if activeTab === 0}<DiskTab bind:state={diskState} bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 1}<RamTab bind:state={ramState} bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 2}<WriteBlockerTab bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 3}<MobileTab bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 4}<CloudTab bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 5}<NetworkTab bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 6}<EncryptionTab bind:state={encryptionState} bind:busy bind:msg {timeoutPromise} />{/if}
    {#if activeTab === 7}<CocTab bind:state={cocState} bind:busy bind:msg {timeoutPromise} />{/if}
  </main>

  {#if msg}
    <div class="toast" class:error={msg.includes("❌")} class:warn={msg.includes("⚠️")}>
      {msg}
      <button class="close-toast" onclick={() => msg = ""}>✕</button>
    </div>
  {/if}
</div>

<style>
:global(body) {
  margin: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #0a0a0a; color: #e0e0e0;
  --card: #141414; --border: #2a2a2a; --primary: #3b82f6;
  --success: #22c55e; --danger: #ef4444; --warn: #f59e0b; --text-secondary: #86868b;
}
.app { max-width: 1100px; margin: 0 auto; padding: 0 16px 40px; }
.header { padding: 20px 0; border-bottom: 1px solid var(--border); margin-bottom: 12px; }
.header h1 { margin: 0; font-size: 22px; }
.ver { font-size: 12px; color: var(--text-secondary); font-weight: normal; }
.subtitle { font-size: 12px; color: var(--text-secondary); }
.tabs { display: flex; gap: 2px; flex-wrap: wrap; margin-bottom: 16px; }
.tab {
  padding: 8px 14px; border: 1px solid var(--border); border-radius: 8px 8px 0 0;
  background: var(--card); color: var(--text-secondary); cursor: pointer;
  font-size: 13px; transition: all 0.15s;
}
.tab:hover { color: #e0e0e0; }
.tab.active { background: #1a1a2e; color: var(--primary); border-bottom-color: transparent; font-weight: 600; }
.content { background: var(--card); border: 1px solid var(--border); border-radius: 0 10px 10px 10px; padding: 20px; min-height: 400px; }
.toast {
  position: fixed; bottom: 20px; right: 20px; padding: 12px 18px;
  border-radius: 10px; background: #1a2e1a; border: 1px solid var(--success);
  font-size: 13px; max-width: 400px; z-index: 1000;
}
.toast.error { background: #2e1a1a; border-color: var(--danger); }
.toast.warn { background: #2e2a1a; border-color: var(--warn); }
.close-toast { background: none; border: none; color: inherit; cursor: pointer; margin-left: 10px; }
</style>