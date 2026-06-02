<script>
import { invoke } from "@tauri-apps/api/core";

let activeTab = $state(0);
let msg = $state("");
let busy = $state(false);

function timeoutPromise(promise, ms) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject("TIMEOUT"), ms);
  });
  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

const tabs = [
  { label: "Compress", icon: "📦" },
  { label: "Extract", icon: "📂" },
  { label: "About", icon: "ℹ️" },
];

const aboutInfo = {
  appName: "ZipLoom",
  version: "0.1.0",
  developer: "YSF Studio — Built with ❤️ by Yusuf Shalahuddin",
  build: "Master Build — All Features Unlocked",
  features: [
    "Drag & Drop Archive Compression & Extraction",
    "Multi-format Support: ZIP, TAR, GZ, BZ2, XZ, 7Z, RAR",
    "AES-256 Encryption for Sensitive Archives",
    "Clean ZIP Output — No macOS Metadata Pollution",
    "100% Offline — Zero Data Collection. All processing runs locally."
  ],
  disclaimer: "This software is provided 'AS-IS'. Results should be independently verified before use in legal proceedings.",
  offline: true,
  privacy: "100% offline — zero data collection. No telemetry, no analytics, no external network calls."
};
</script>

<div class="app-shell">
  <div class="titlebar">
    <div class="traffic-lights">
      <span class="tl red"></span><span class="tl yellow"></span><span class="tl green"></span>
    </div>
    <img src="/src-tauri/icons/logo.svg" class="logo" alt="ZipLoom" />
    <span class="title">ZipLoom</span>
    <span class="version">v0.1.0</span>
    <div class="tabstrip">
      {#each tabs as tab, i}
        <button class:active={activeTab === i} onclick={() => activeTab = i}>
          {tab.icon} {tab.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="workspace">
    <main class="workspace-main">
      {#if activeTab === 0}
        <div class="card" style="max-width:600px;margin:0 auto">
          <h3>📦 Compress Files</h3>
          <p style="color:var(--text-secondary);font-size:13px;margin-bottom:16px">
            Drag & drop files or folders to create a compressed archive.
            Supports ZIP, TAR, GZ, BZ2, XZ, and 7Z formats.
          </p>
          <div class="dropzone">
            <span style="font-size:32px">📁</span>
            <p>Drop files here</p>
            <span style="font-size:11px;color:var(--text-muted)">or click to browse</span>
          </div>
        </div>
      {:else if activeTab === 1}
        <div class="card" style="max-width:600px;margin:0 auto">
          <h3>📂 Extract Archives</h3>
          <p style="color:var(--text-secondary);font-size:13px;margin-bottom:16px">
            Drag & drop an archive to extract its contents.
            Supports ZIP, RAR, 7Z, TAR, GZ, BZ2, XZ, and ZST formats.
          </p>
          <div class="dropzone">
            <span style="font-size:32px">📦</span>
            <p>Drop archive here</p>
            <span style="font-size:11px;color:var(--text-muted)">or click to browse</span>
          </div>
        </div>
      {:else if activeTab === 2}
        <div style="max-width:580px;margin:0 auto">
          <!-- About content -->
          <div style="text-align:center;margin-bottom:24px">
            <div style="font-size:48px;margin-bottom:8px">🗜️</div>
            <h3 style="margin:0 0 4px">{aboutInfo.appName}<span style="color:var(--text-muted);font-size:12px;margin-left:8px">v{aboutInfo.version}</span></h3>
            <p style="color:var(--text-secondary);font-size:13px;margin:0">Archive Utility — Fast, Clean, Offline</p>
          </div>

          <div class="card" style="margin-bottom:12px">
            <h4>Features</h4>
            <ul style="margin:0;padding-left:20px">
              {#each aboutInfo.features as f}
                <li style="font-size:13px;color:var(--text-secondary);margin-bottom:6px;line-height:1.4">{f}</li>
              {/each}
            </ul>
          </div>

          <div class="card" style="margin-bottom:12px;border-left:3px solid var(--success)">
            <h4>🔒 Privacy & Security</h4>
            <p style="font-size:13px;color:var(--text-secondary);margin:0;line-height:1.5">{aboutInfo.privacy}</p>
            <span class="offline-badge" style="margin-top:8px">✅ Fully Offline</span>
          </div>

          <div class="card" style="margin-bottom:12px;border-left:3px solid var(--warn)">
            <h4>⚖️ Disclaimer</h4>
            <p style="font-size:13px;color:var(--text-secondary);margin:0;font-style:italic">{aboutInfo.disclaimer}</p>
          </div>

          <div class="card">
            <h4>👨‍💻 Developer</h4>
            <p style="font-size:13px;color:var(--text);margin:0 0 4px;font-weight:600">{aboutInfo.developer}</p>
            <p style="font-size:12px;color:var(--primary);margin:0">{aboutInfo.build}</p>
          </div>

          <p style="text-align:center;font-size:11px;color:var(--text-muted);margin-top:12px">
            YSF Studio © {new Date().getFullYear()} — All rights reserved.
          </p>
        </div>
      {/if}
    </main>
  </div>

  <div class="statusbar">
    <div class="sb-left">
      <span class="status-dot"></span> ZipLoom — Archive Utility
    </div>
    <div class="sb-center"></div>
    <div class="sb-right">
      <span class="offline-badge">🔒 Offline</span>
    </div>
  </div>
</div>

<style>
.dropzone {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 180px; border: 2px dashed var(--border); border-radius: var(--radius-lg);
  background: var(--card-hover); cursor: pointer; gap: 8px; transition: border-color 0.2s;
}
.dropzone:hover { border-color: var(--primary); }
.dropzone p { margin: 0; font-size: 14px; color: var(--text-secondary); }
</style>
