<script>
import { invoke } from "@tauri-apps/api/core";
import DiskTab from "./lib/components/DiskTab.svelte";
import RamTab from "./lib/components/RamTab.svelte";
import WriteBlockerTab from "./lib/components/WriteBlockerTab.svelte";
import MobileTab from "./lib/components/MobileTab.svelte";
import CloudTab from "./lib/components/CloudTab.svelte";
import NetworkTab from "./lib/components/NetworkTab.svelte";
import EncryptionTab from "./lib/components/EncryptionTab.svelte";
import VerificationTab from "./lib/components/VerificationTab.svelte";
import CocTab from "./lib/components/CocTab.svelte";
import SnapshotTab from "./lib/components/SnapshotTab.svelte";
import DisclaimerTab from "./lib/components/DisclaimerTab.svelte";

let activeSection = $state("disk");
let msg = $state("");
let busy = $state(false);
let wbActive = $state(false);
let progress = $state(null);

// Shared state — plain objects, no $state() to avoid Svelte 5 Proxy subscribe pattern in WebKit
let diskState = {};
let ramState = {};
let encryptionState = {};
let cocState = {};
let wbState = { active: false };

function timeoutPromise(promise, ms) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject("TIMEOUT"), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

function setBusy(v) { busy = v; }
function setMsg(m) { msg = m; }

const sidebarSections = [
  {
    label: "ACQUISITION",
    items: [
      { id: "disk", icon: "💿", label: "Disk Imaging" },
      { id: "ram", icon: "🧠", label: "RAM Capture" },
      { id: "mobile", icon: "📱", label: "Mobile Triage" },
      { id: "cloud", icon: "☁️", label: "Cloud Snapshot" },
      { id: "network", icon: "🌐", label: "Network Capture" },
      { id: "snapshot", icon: "📸", label: "System Snapshot" },
    ]
  },
  {
    label: "ANALYSIS",
    items: [
      { id: "encryption", icon: "🔐", label: "Encryption" },
      { id: "verify", icon: "🔍", label: "Hash Verify" },
    ]
  },
  {
    label: "CASE INFO",
    items: [
      { id: "coc", icon: "📋", label: "Custody Chain" },
      { id: "about", icon: "ℹ️", label: "About" },
    ]
  }
];

$effect(() => {
  if (wbState.active !== wbActive) wbActive = wbState.active;
});
</script>

<div class="app-shell">
  <!-- macOS Unified Titlebar -->
  <div class="titlebar">
    <div class="traffic-lights">
      <span class="tl red"></span><span class="tl yellow"></span><span class="tl green"></span>
    </div>
    <div class="titlebar-center">
      <img src="/src-tauri/icons/logo.svg" class="logo" alt="CL" />
      <span class="title">CollectionLoom</span>
      {#if wbState.active}
        <span class="pill-badge on">🛡️ Write-Blocker Active</span>
      {:else}
        <span class="pill-badge off">Write-Blocker Inactive</span>
      {/if}
    </div>
    <div class="titlebar-end">
      <div class="titlebar-end"></div>
    </div>
  </div>
  <div class="two-pane">
    <aside class="sidebar">
      {#each sidebarSections as section}
        <div class="sidebar-group">
          <span class="sidebar-label">{section.label}</span>
          {#each section.items as item}
            <button class="sidebar-item" class:active={activeSection === item.id}
              onclick={() => activeSection = item.id}>
              {item.icon} {item.label}
            </button>
          {/each}
        </div>
      {/each}
    </aside>

    <!-- Content Area -->
    <div class="content-area">
      {#if activeSection === "disk"}
        <DiskTab state={diskState} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "ram"}
        <RamTab state={ramState} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "mobile"}
        <MobileTab {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "cloud"}
        <CloudTab {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "network"}
        <NetworkTab {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "snapshot"}
        <SnapshotTab {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "encryption"}
        <EncryptionTab state={encryptionState} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "verify"}
        <VerificationTab {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "coc"}
        <CocTab state={cocState} {setBusy} {setMsg} {timeoutPromise} />
      {:else if activeSection === "about"}
        <DisclaimerTab />
      {/if}
    </div>
  </div>

  <!-- Status Bar -->
  <div class="statusbar">
    <div class="sb-left">
      <span class="status-dot" class:on={wbState.active} class:busy={busy}></span>
      {#if busy}
        <span class="spinner">⏳</span> Processing...
      {:else}
        CollectionLoom — Forensic Acquisition Toolkit
      {/if}
    </div>
    <div class="sb-right">
      <span class="offline-badge">🔒 Offline</span>
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

.titlebar {
  display: flex; align-items: center;
  height: 44px; padding: 0 14px; gap: 12px;
  background: rgba(20,20,20,0.95);
  backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255,255,255,0.06);
  -webkit-app-region: drag;
}
.traffic-lights { display: flex; gap: 7px; -webkit-app-region: no-drag; }
.tl { width: 12px; height: 12px; border-radius: 50%; }
.tl.red { background: #ff5f57; } .tl.yellow { background: #ffbd2e; } .tl.green { background: #28c840; }
.titlebar-center { display: flex; align-items: center; gap: 8px; flex: 1; -webkit-app-region: no-drag; }
.titlebar-end { -webkit-app-region: no-drag; }
.logo { width: 18px; height: 18px; border-radius: 3px; }
.title { font-size: 13px; font-weight: 600; color: #ccc; }
.pill-badge { padding: 1px 10px; border-radius: 10px; font-size: 10px; font-weight: 600; }
.pill-badge.on { background: rgba(34,197,94,0.12); color: #22c55e; }
.pill-badge.off { background: rgba(239,68,68,0.08); color: #ef4444; }

.two-pane { display: flex; flex: 1; overflow: hidden; }

.sidebar {
  width: 200px; min-width: 200px;
  background: rgba(12,12,12,0.8); backdrop-filter: blur(20px);
  border-right: 1px solid rgba(255,255,255,0.06); overflow-y: auto; padding: 8px 0;
}
.sidebar-group { margin-bottom: 4px; }
.sidebar-label { display: block; padding: 6px 14px 3px; font-size: 10px; font-weight: 700; color: #555; text-transform: uppercase; letter-spacing: 0.5px; }
.sidebar-item {
  display: flex; align-items: center; gap: 7px; width: calc(100% - 16px); padding: 5px 14px; margin: 0 8px;
  border: none; border-radius: 5px; background: transparent; color: #999; cursor: pointer;
  font-size: 12px; text-align: left; transition: all 0.12s;
}
.sidebar-item:hover { background: rgba(255,255,255,0.04); color: #ccc; }
.sidebar-item.active { background: rgba(59,130,246,0.12); color: #3b82f6; font-weight: 600; }

.content-area { flex: 1; overflow-y: auto; padding: 24px; }

.statusbar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 14px; height: 26px;
  background: rgba(10,10,10,0.95); border-top: 1px solid rgba(255,255,255,0.06);
  font-size: 11px; color: #555;
}
.sb-left, .sb-right { display: flex; align-items: center; gap: 6px; }
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: #444; }
.status-dot.on { background: #22c55e; box-shadow: 0 0 3px #22c55e; }
.status-dot.busy { background: #f59e0b; animation: pulse 1s infinite; }
@keyframes pulse { 0%,100%{ opacity:1; } 50%{ opacity:0.3; } }
.offline-badge { padding: 0 6px; background: rgba(34,197,94,0.1); color: #22c55e; border-radius: 8px; font-size: 10px; font-weight: 600; }
.spinner { display: inline-block; animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.toast {
  position: fixed; bottom: 44px; right: 20px; padding: 10px 16px; border-radius: 10px;
  background: #1a2e1a; border: 1px solid #22c55e; font-size: 12px; max-width: 380px; z-index: 1000;
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
