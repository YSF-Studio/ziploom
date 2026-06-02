<script>
import { invoke } from "@tauri-apps/api/core";
import CaseTab from "./lib/components/CaseTab.svelte";
import FileBrowserTab from "./lib/components/FileBrowserTab.svelte";
import CarvingTab from "./lib/components/CarvingTab.svelte";
import TimelineTab from "./lib/components/TimelineTab.svelte";
import SearchTab from "./lib/components/SearchTab.svelte";
import DisclaimerTab from "./lib/components/DisclaimerTab.svelte";
import InspectorPanel from "./lib/components/InspectorPanel.svelte";

let activeView = $state("files");
let msg = $state("");
let busy = $state(false);
let activeCase = $state(null);
let selectedFile = $state(null);
let inspectorMeta = $state(null);
let searchQuery = $state("");
let density = $state("compact");

function timeoutPromise(promise, ms) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject("TIMEOUT"), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

const sidebarSections = [
  {
    label: "SOURCES",
    items: [
      { id: "cases", icon: "📁", label: "Case Manager" },
      { id: "files", icon: "🗂️", label: "File Browser" },
    ]
  },
  {
    label: "VIEWS",
    items: [
      { id: "timeline", icon: "📊", label: "Timeline" },
      { id: "carving", icon: "🔍", label: "Carved Files" },
      { id: "search", icon: "🔎", label: "Search" },
    ]
  },
  {
    label: "INFO",
    items: [
      { id: "about", icon: "ℹ️", label: "About" },
    ]
  }
];

function onFileSelect(path, meta) {
  selectedFile = path;
  if (meta) inspectorMeta = meta;
}

function handleSearchSubmit() {
  if (searchQuery.trim()) {
    activeView = "search";
  }
}
</script>

<div class="app-shell">
  <!-- macOS Unified Titlebar -->
  <div class="titlebar">
    <div class="traffic-lights">
      <span class="tl red"></span><span class="tl yellow"></span><span class="tl green"></span>
    </div>
    <div class="titlebar-nav">
      <button class="nav-btn" disabled>‹</button>
      <button class="nav-btn" disabled>›</button>
    </div>
    <img src="/src-tauri/icons/logo.svg" class="logo" alt="AL" />
    <span class="title">AnalysisLoom</span>

    <!-- Center Search Bar -->
    <div class="search-bar">
      <span class="search-icon">🔍</span>
      <input type="text" placeholder="Keyword / Regex search..."
        bind:value={searchQuery}
        onkeydown={(e) => e.key === "Enter" && handleSearchSubmit()}
      />
      {#if searchQuery}
        <button class="search-clear" onclick={() => searchQuery = ""}>✕</button>
      {/if}
    </div>

    <div class="titlebar-end">
      {#if activeView === "files"}
        <button class="toolbar-btn" onclick={() => {
          const d = ["compact","standard","comfortable"];
          density = d[(d.indexOf(density) + 1) % 3];
        }}>≡</button>
      {/if}
    </div>
  </div>

  <!-- Three-Pane Layout -->
  <div class="three-pane">
    <!-- Left Sidebar -->
    <aside class="sidebar">
      {#each sidebarSections as section}
        <div class="sidebar-group">
          <span class="sidebar-label">{section.label}</span>
          {#each section.items as item}
            <button class="sidebar-item" class:active={activeView === item.id}
              onclick={() => activeView = item.id}>
              {item.icon} {item.label}
            </button>
          {/each}
        </div>
      {/each}
    </aside>

    <!-- Center: Main Content -->
    <div class="main-content">
      {#if activeView === "cases"}
        <CaseTab bind:activeCase bind:busy bind:msg {timeoutPromise} />
      {:else if activeView === "files"}
        <FileBrowserTab bind:activeCase bind:busy bind:msg {timeoutPromise} {density} {onFileSelect} />
      {:else if activeView === "timeline"}
        <TimelineTab bind:activeCase bind:busy bind:msg {timeoutPromise} />
      {:else if activeView === "carving"}
        <CarvingTab bind:activeCase bind:busy bind:msg {timeoutPromise} />
      {:else if activeView === "search"}
        <SearchTab bind:activeCase bind:busy bind:msg {timeoutPromise} />
      {:else if activeView === "about"}
        <DisclaimerTab />
      {/if}
    </div>

    <!-- Right Inspector -->
    {#if inspectorMeta || selectedFile}
      <aside class="inspector-pane">
        <div class="inspector-head">
          <span>📋 Inspector</span>
          <button class="inspector-close" onclick={() => { inspectorMeta = null; selectedFile = null; }}>✕</button>
        </div>
        <InspectorPanel metadata={inspectorMeta} visible={true} />
      </aside>
    {/if}
  </div>

  <!-- Status Bar -->
  <div class="statusbar">
    <div class="sb-left">
      <span class="status-dot" class:on={!!activeCase} class:busy={busy}></span>
      {activeCase?.name || "AnalysisLoom"}
      {#if selectedFile}
        <span style="opacity:0.4;margin:0 6px">›</span>
        <span class="file-path" title={selectedFile}>
          {selectedFile.split("/").pop() || selectedFile}
        </span>
      {/if}
    </div>
    <div class="sb-right">
      <span class="offline-badge">🔒 Offline</span>
      <span>ISO 27042</span>
    </div>
  </div>

  {#if msg}
    <div class="toast" class:error={msg.includes("❌")} class:warn={msg.includes("⚠️")}>
      {msg}
      <button class="close-toast" onclick={() => msg = ""}>✕</button>
    </div>
  {/if}
</div>

<style>
:global(body) {
  margin: 0; padding: 0; overflow: hidden; height: 100vh;
  background: #0a0a0a; color: #e0e0e0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
}
:global(*), :global(*::before), :global(*::after) { box-sizing: border-box; }

.app-shell { display: flex; flex-direction: column; height: 100vh; }

/* Titlebar */
.titlebar {
  display: flex; align-items: center;
  height: 44px; padding: 0 10px; gap: 8px;
  background: rgba(20,20,20,0.95);
  backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255,255,255,0.06);
  -webkit-app-region: drag;
}
.traffic-lights { display: flex; gap: 7px; -webkit-app-region: no-drag; }
.tl { width: 12px; height: 12px; border-radius: 50%; }
.tl.red { background: #ff5f57; } .tl.yellow { background: #ffbd2e; } .tl.green { background: #28c840; }
.titlebar-nav { display: flex; gap: 2px; -webkit-app-region: no-drag; }
.nav-btn {
  width: 26px; height: 24px; border: none; border-radius: 4px;
  background: transparent; color: #666; font-size: 16px; cursor: pointer;
  display: flex; align-items: center; justify-content: center;
}
.nav-btn:hover { background: rgba(255,255,255,0.08); color: #aaa; }
.nav-btn:disabled { opacity: 0.3; cursor: default; }
.logo { width: 18px; height: 18px; border-radius: 3px; -webkit-app-region: no-drag; }
.title { font-size: 13px; font-weight: 600; color: #ccc; -webkit-app-region: no-drag; }

/* Center Search Bar */
.search-bar {
  display: flex; align-items: center; flex: 1; max-width: 400px; margin: 0 auto;
  background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);
  border-radius: 6px; padding: 0 10px; height: 28px;
  -webkit-app-region: no-drag;
}
.search-bar:focus-within { border-color: rgba(59,130,246,0.4); }
.search-icon { font-size: 11px; opacity: 0.4; margin-right: 6px; }
.search-bar input {
  flex: 1; background: transparent; border: none; color: #ccc;
  font-size: 12px; outline: none; padding: 0;
}
.search-bar input::placeholder { color: #555; }
.search-clear {
  background: none; border: none; color: #666; cursor: pointer;
  font-size: 11px; padding: 2px 4px;
}
.search-clear:hover { color: #aaa; }

.titlebar-end { display: flex; gap: 4px; -webkit-app-region: no-drag; }
.toolbar-btn {
  width: 28px; height: 24px; border: none; border-radius: 4px;
  background: transparent; color: #888; font-size: 14px;
  cursor: pointer; display: flex; align-items: center; justify-content: center;
}
.toolbar-btn:hover { background: rgba(255,255,255,0.08); color: #ccc; }

/* Three-Pane */
.three-pane { display: flex; flex: 1; overflow: hidden; }

/* Sidebar */
.sidebar {
  width: 190px; min-width: 190px;
  background: rgba(12,12,12,0.8);
  backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
  border-right: 1px solid rgba(255,255,255,0.06);
  overflow-y: auto; padding: 6px 0;
}
.sidebar-group { margin-bottom: 4px; }
.sidebar-label {
  display: block; padding: 6px 14px 3px;
  font-size: 10px; font-weight: 700; color: #555;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.sidebar-item {
  display: flex; align-items: center; gap: 7px;
  width: calc(100% - 16px); padding: 5px 14px; margin: 0 8px;
  border: none; border-radius: 5px;
  background: transparent; color: #999; cursor: pointer;
  font-size: 12px; text-align: left; transition: all 0.12s;
}
.sidebar-item:hover { background: rgba(255,255,255,0.04); color: #ccc; }
.sidebar-item.active { background: rgba(59,130,246,0.12); color: #3b82f6; font-weight: 600; }

/* Main Content */
.main-content {
  flex: 1; overflow-y: auto; padding: 16px 20px;
  border-right: 1px solid rgba(255,255,255,0.04);
}

/* Inspector Pane */
.inspector-pane {
  width: 280px; min-width: 280px;
  background: rgba(10,10,10,0.9);
  backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
  border-left: 1px solid rgba(255,255,255,0.06);
  overflow-y: auto; display: flex; flex-direction: column;
}
.inspector-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 14px; border-bottom: 1px solid rgba(255,255,255,0.06);
  font-size: 12px; color: #aaa;
}
.inspector-close {
  background: none; border: none; color: #555; cursor: pointer;
  font-size: 13px; padding: 2px 4px;
}
.inspector-close:hover { color: #aaa; }

/* Status Bar */
.statusbar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 14px; height: 26px;
  background: rgba(10,10,10,0.95);
  backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
  border-top: 1px solid rgba(255,255,255,0.06);
  font-size: 11px; color: #555;
}
.sb-left, .sb-right { display: flex; align-items: center; gap: 6px; }
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: #444; }
.status-dot.on { background: #22c55e; box-shadow: 0 0 3px #22c55e; }
.status-dot.busy { background: #f59e0b; animation: pulse 1s infinite; }
@keyframes pulse { 0%,100%{ opacity:1; } 50%{ opacity:0.3; } }
.offline-badge {
  padding: 0 6px; background: rgba(34,197,94,0.1); color: #22c55e;
  border-radius: 8px; font-size: 10px; font-weight: 600;
}
.file-path {
  max-width: 200px; overflow: hidden; text-overflow: ellipsis;
  white-space: nowrap; font-family: "SF Mono", Menlo, monospace; font-size: 10px;
}
@keyframes spin { to { transform: rotate(360deg); } }
.spinner { display: inline-block; animation: spin 1s linear infinite; }

/* Toast */
.toast {
  position: fixed; bottom: 44px; right: 20px;
  padding: 10px 16px; border-radius: 10px;
  background: #1a2e1a; border: 1px solid #22c55e;
  font-size: 12px; max-width: 380px; z-index: 1000;
  animation: slideUp 0.2s ease-out;
}
.toast.error { background: #2e1a1a; border-color: #ef4444; }
.toast.warn { background: #2e2a1a; border-color: #f59e0b; }
@keyframes slideUp { from{opacity:0;transform:translateY(8px)} to{opacity:1;transform:translateY(0)} }
.close-toast { background: none; border: none; color: inherit; cursor: pointer; margin-left: 10px; }

::-webkit-scrollbar { width: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 3px; }
</style>
