<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import Logo from "./lib/Logo.svelte";

  let activeTab = $state(0);
  let msg = $state("");
  let busy = $state(false);

  // ─── Compress State ───
  let compSources = $state([]);
  let compFormat = $state("zip");
  let compResult = $state(null);

  // ─── Extract State ───
  let extrArchive = $state("");
  let extrResult = $state(null);

  // ─── Inspect State ───
  let inspArchive = $state("");
  let inspInfo = $state(null);
  let inspError = $state("");

  // ─── Encrypt State ───
  let encFilePath = $state("");
  let encPassword = $state("");
  let encMode = $state("encrypt"); // "encrypt" or "decrypt"
  let encResult = $state(null);

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
    { label: "Inspect", icon: "🔍" },
    { label: "About", icon: "ℹ️" },
    { label: "Encrypt", icon: "🔐" },
  ];

  const formats = [
    { value: "zip", label: "ZIP (.zip)" },
    { value: "tar", label: "TAR (.tar)" },
    { value: "tar.gz", label: "TAR.GZ (.tar.gz)" },
  ];

  // ─── Auto-dismiss success messages ───
  $effect(() => {
    if (msg && !msg.startsWith("❌")) {
      const t = setTimeout(() => msg = "", 8000);
      return () => clearTimeout(t);
    }
  });

  // ─── Drag & Drop ───
  async function onDrop(e) {
    e.preventDefault();
    if (busy) return;
    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;
    const paths = [];
    for (const f of files) {
      if (f.path) paths.push(f.path);
    }
    if (paths.length === 0) return;

    if (activeTab === 0) {
      compSources = [...compSources, ...paths];
    } else if (activeTab === 1) {
      extrArchive = paths[0];
      await doExtract(paths[0]);
    } else if (activeTab === 2) {
      inspArchive = paths[0];
      await doInspect(paths[0]);
    } else if (activeTab === 4) {
      encFilePath = paths[0];
    }
  }

  // ─── Compress: Browse Files ───
  async function browseSources() {
    if (busy) return;
    try {
      const selected = await open({ directory: false, multiple: true });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        compSources = [...compSources, ...paths];
      }
    } catch (e) {
      msg = `❌ ${typeof e === 'string' ? e : String(e)}`;
    }
  }

  async function browseDir() {
    if (busy) return;
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        compSources = [...compSources, selected];
      }
    } catch (e) {
      msg = `❌ ${typeof e === 'string' ? e : String(e)}`;
    }
  }

  function removeSource(i) {
    compSources = compSources.filter((_, idx) => idx !== i);
  }

  async function doCompress() {
    if (compSources.length === 0) {
      msg = "❌ Add files or folders first";
      return;
    }
    busy = true;
    compResult = null;
    try {
      const output = await open({
        directory: false,
        multiple: false,
        title: "Save archive as",
        defaultPath: `archive.${compFormat === 'tar.gz' ? 'tar.gz' : compFormat}`,
      });
      if (!output) { busy = false; return; }
      const result = await timeoutPromise(
        invoke("compress_files", { sources: compSources, output, format: compFormat }),
        120000
      );
      compResult = result;
      msg = `✅ Compressed ${result.filesProcessed} files (${(result.totalSize / 1024).toFixed(1)} KB)`;
    } catch (e) {
      const err = typeof e === 'string' ? e : String(e);
      if (err === "TIMEOUT") msg = "❌ Compression timed out — try fewer/smaller files";
      else msg = `❌ ${err}`;
    }
    busy = false;
  }

  // ─── Extract: Browse Archive ───
  async function browseArchive() {
    if (busy) return;
    try {
      const selected = await open({ directory: false, multiple: false });
      if (selected) {
        extrArchive = selected;
        await doExtract(selected);
      }
    } catch (e) {
      msg = `❌ ${typeof e === 'string' ? e : String(e)}`;
    }
  }

  async function doExtract(src) {
    busy = true;
    extrResult = null;
    try {
      const dir = await open({ directory: true, multiple: false, title: "Choose extraction folder" });
      if (!dir) { busy = false; return; }
      const result = await timeoutPromise(
        invoke("extract_archive", { archivePath: src, outputDir: dir }),
        120000
      );
      extrResult = result;
      msg = `✅ Extracted ${result.filesProcessed} files (${(result.totalSize / 1024).toFixed(1)} KB) to ${dir}`;
    } catch (e) {
      const err = typeof e === 'string' ? e : String(e);
      if (err === "TIMEOUT") msg = "❌ Extraction timed out";
      else msg = `❌ ${err}`;
    }
    busy = false;
  }

  // ─── Inspect: Browse Archive ───
  async function browseInspect() {
    if (busy) return;
    try {
      const selected = await open({ directory: false, multiple: false });
      if (selected) {
        inspArchive = selected;
        await doInspect(selected);
      }
    } catch (e) {
      msg = `❌ ${typeof e === 'string' ? e : String(e)}`;
    }
  }

  async function doInspect(src) {
    busy = true;
    inspInfo = null;
    inspError = "";
    try {
      const info = await timeoutPromise(
        invoke("inspect_archive", { path: src }),
        30000
      );
      inspInfo = info;
    } catch (e) {
      inspInfo = null;
      inspError = typeof e === 'string' ? e : String(e);
      if (inspError === "TIMEOUT") inspError = "Inspection timed out — archive may be corrupted";
    }
    busy = false;
  }

  // ─── Encrypt: Browse File ───
  async function browseEncFile() {
    if (busy) return;
    try {
      const selected = await open({ directory: false, multiple: false });
      if (selected) {
        encFilePath = selected;
      }
    } catch (e) {
      msg = `❌ ${typeof e === 'string' ? e : String(e)}`;
    }
  }

  async function doEncrypt() {
    if (!encFilePath) {
      msg = "❌ Select a file first";
      return;
    }
    if (!encPassword) {
      msg = "❌ Enter a password";
      return;
    }
    busy = true;
    encResult = null;
    try {
      const cmd = encMode === "encrypt" ? "encrypt_file" : "decrypt_file";
      const result = await timeoutPromise(
        invoke(cmd, { path: encFilePath, password: encPassword }),
        120000
      );
      encResult = result;
      const action = encMode === "encrypt" ? "Encrypted" : "Decrypted";
      msg = `✅ ${action}: ${result}`;
    } catch (e) {
      const err = typeof e === 'string' ? e : String(e);
      if (err === "TIMEOUT") msg = `❌ ${encMode === "encrypt" ? "Encryption" : "Decryption"} timed out`;
      else msg = `❌ ${err}`;
    }
    busy = false;
  }

  function clearEnc() {
    encFilePath = "";
    encPassword = "";
    encResult = null;
  }

  function formatSize(bytes) {
    if (bytes == null) return "—";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  function compRatio(entry) {
    if (!entry.compressedSize || entry.size === 0) return null;
    const ratio = ((1 - entry.compressedSize / entry.size) * 100);
    return ratio.toFixed(0);
  }

  // About info (static, no invoke needed)
  const aboutInfo = {
    appName: "ZipLoom",
    version: "0.1.0",
    developer: "YSF Studio — Built with ❤️ by Yusuf Shalahuddin",
    build: "Master Build — All Features Unlocked",
    features: [
      "Drag & Drop Archive Compression & Extraction",
      "Multi-format Support: ZIP, TAR, GZ, BZ2, XZ, 7Z, RAR",
      "Archive Inspector — Preview contents without extracting",
      "AES-256 Encryption for Sensitive Archives",
      "Clean ZIP Output — No macOS Metadata Pollution",
      "100% Offline — Zero Data Collection. All processing runs locally."
    ],
    disclaimer: "This software is provided 'AS-IS'. Results should be independently verified before use in legal proceedings.",
    offline: true,
    privacy: "100% offline — zero data collection. No telemetry, no analytics, no external network calls."
  };

  // Register drop handler on window
  let unlisten = $state();
  $effect(() => {
    const win = getCurrentWebviewWindow();
    win.onDragDropEvent(async (event) => {
      if (event.payload.type === 'drop') {
        const paths = event.payload.paths;
        if (paths && paths.length > 0 && !busy) {
          if (activeTab === 0) {
            compSources = [...compSources, ...paths];
          } else if (activeTab === 1) {
            extrArchive = paths[0];
            await doExtract(paths[0]);
          } else if (activeTab === 2) {
            inspArchive = paths[0];
            await doInspect(paths[0]);
          } else if (activeTab === 4) {
            encFilePath = paths[0];
          }
        }
      }
    });
  });

  // ─── Expose helpers for GUI screenshot mode ───
  if (typeof window !== 'undefined') {
    window.__zipLoom = {
      setTab: (i) => { activeTab = i; msg = ''; },
      setSources: (paths) => { compSources = paths; },
      setInspectResult: (data) => { inspInfo = data; inspError = ''; },
      setExtractResult: (archive) => { extrArchive = archive; },
      setMsg: (m) => { msg = m; },
      toggleLight: () => {
        document.documentElement.classList.toggle('light-mode');
      },
    };
  }
</script>

<div class="app-shell">
  <div class="titlebar">
    <div class="traffic-lights">
      <span class="tl red"></span><span class="tl yellow"></span><span class="tl green"></span>
    </div>
    <Logo size={22} />
    <span class="title">ZipLoom</span>
    <span class="version">v0.1.0</span>
    <div class="tabstrip">
      {#each tabs as tab, i}
        <button class:active={activeTab === i} onclick={() => { activeTab = i; msg = ""; }}>
          {tab.icon} {tab.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="workspace">
    <main class="workspace-main">
      {#if msg}
        <div class="toast" class:error={msg.startsWith("❌")}>{msg}</div>
      {/if}

      <!-- ─── COMPRESS TAB ─── -->
      {#if activeTab === 0}
        <div class="tab-content">
          <h3>📦 Compress Files</h3>
          <p class="subtitle">
            Select files and folders, choose a format, then compress into an archive.
          </p>

          <!-- Source list -->
          {#if compSources.length > 0}
            <div class="source-list">
              <h4>Selected ({compSources.length} item{compSources.length > 1 ? 's' : ''})</h4>
              {#each compSources as src, i}
                <div class="source-item">
                  <span class="src-icon">{src.endsWith('/') || src.includes('/.') ? '📁' : '📄'}</span>
                  <span class="src-path">{src}</span>
                  <button class="btn-remove" onclick={() => removeSource(i)}>✕</button>
                </div>
              {/each}
            </div>
          {/if}

          <!-- Format selector -->
          <div class="format-row">
            <label for="format-select">Format:</label>
            <select id="format-select" bind:value={compFormat} disabled={busy}>
              {#each formats as f}
                <option value={f.value}>{f.label}</option>
              {/each}
            </select>
          </div>

          <!-- Add buttons -->
          <div class="btn-row">
            <button class="btn" onclick={browseDir} disabled={busy}>📁 Add Folder</button>
            <button class="btn" onclick={browseSources} disabled={busy}>📄 Add Files</button>
          </div>

          <!-- Compress button -->
          <button class="btn-primary" onclick={doCompress} disabled={busy || compSources.length === 0}>
            {busy ? '🔄 Compressing...' : '🗜️ Compress'}
          </button>

          <!-- Result -->
          {#if compResult}
            <div class="result-card success">
              <span class="result-icon">✅</span>
              <div>
                <strong>{compResult.message}</strong><br />
                <span class="muted">{compResult.filesProcessed} files → {compResult.outputPath}</span>
              </div>
            </div>
          {/if}

          <!-- Empty state with drop zone -->
          {#if compSources.length === 0 && !busy}
            <div class="dropzone" role="button" tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onDrop}>
              <span style="font-size:32px">📁</span>
              <p>Drop files or folders here</p>
              <span class="hint">or use the buttons above to browse</span>
            </div>
          {/if}
        </div>

      <!-- ─── EXTRACT TAB ─── -->
      {:else if activeTab === 1}
        <div class="tab-content">
          <h3>📂 Extract Archives</h3>
          <p class="subtitle">
            Select an archive to extract its contents. Supports ZIP, TAR, GZ, BZ2, XZ.
          </p>

          {#if extrArchive}
            <div class="source-item">
              <span class="src-icon">📦</span>
              <span class="src-path">{extrArchive}</span>
            </div>
          {/if}

          <button class="btn-primary" onclick={browseArchive} disabled={busy}>
            {busy ? '🔄 Extracting...' : '📦 Choose Archive & Extract'}
          </button>

          {#if extrResult}
            <div class="result-card success">
              <span class="result-icon">✅</span>
              <div>
                <strong>{extrResult.message}</strong><br />
                <span class="muted">Output: {extrResult.outputPath}</span>
              </div>
            </div>
          {/if}

          {#if !extrArchive && !busy}
            <div class="dropzone" role="button" tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onDrop}>
              <span style="font-size:32px">📦</span>
              <p>Drop archive here</p>
              <span class="hint">or click the button above to browse</span>
            </div>
          {/if}
        </div>

      <!-- ─── INSPECT TAB ─── -->
      {:else if activeTab === 2}
        <div class="tab-content">
          <h3>🔍 Inspect Archive</h3>
          <p class="subtitle">
            Preview archive contents without extracting. View file listing, sizes, and compression ratios.
          </p>

          {#if inspArchive}
            <div class="source-item">
              <span class="src-icon">🗄️</span>
              <span class="src-path">{inspArchive}</span>
            </div>
          {/if}

          <button class="btn-primary" onclick={browseInspect} disabled={busy}>
            {busy ? '🔍 Scanning...' : '🗄️ Choose Archive to Inspect'}
          </button>

          {#if inspError}
            <div class="result-card error">
              <span class="result-icon">❌</span>
              <span>{inspError}</span>
            </div>
          {/if}

          {#if inspInfo}
            <div class="result-card info">
              <span class="result-icon">📋</span>
              <div>
                <strong>{inspInfo.format}</strong> — {inspInfo.totalFiles} files, {formatSize(inspInfo.totalSize)}
                {#if inspInfo.totalCompressed}
                  <span class="muted"> (compressed: {formatSize(inspInfo.totalCompressed)})</span>
                {/if}
              </div>
            </div>

            {#if inspInfo.entries.length > 0}
              <div class="entry-table">
                <div class="entry-header">
                  <span>Name</span>
                  <span>Size</span>
                  <span>Ratio</span>
                </div>
                {#each inspInfo.entries as entry}
                  <div class="entry-row">
                    <span class="entry-name">
                      {entry.isDir ? '📁' : '📄'} {entry.path}
                    </span>
                    <span class="entry-size">{formatSize(entry.size)}</span>
                    <span class="entry-ratio" class:good={compRatio(entry) > 0} class:bad={compRatio(entry) < 0}>
                      {compRatio(entry) != null ? `${compRatio(entry) > 0 ? '−' : '+'}${Math.abs(compRatio(entry))}%` : '—'}
                    </span>
                  </div>
                {/each}
              </div>
            {/if}
          {/if}

          {#if !inspArchive && !busy}
            <div class="dropzone" role="button" tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onDrop}>
              <span style="font-size:32px">🗄️</span>
              <p>Drop archive to inspect</p>
              <span class="hint">or click the button above to browse</span>
            </div>
          {/if}
        </div>

      <!-- ─── ABOUT TAB ─── -->
      {:else if activeTab === 3}
        <div style="max-width:580px;margin:0 auto">
          <div style="text-align:center;margin-bottom:20px">
            <img src="/icon.png" style="width:72px;height:72px;border-radius:16px;margin-bottom:8px" alt="ZipLoom" />
            <h3 style="margin:0 0 4px">{aboutInfo.appName}<span style="color:var(--text-muted);font-size:12px;margin-left:8px">v{aboutInfo.version}</span></h3>
            <p style="color:var(--text-secondary);font-size:13px;margin:0">Archive Utility — Fast, Clean, Offline</p>
          </div>
          <div class="card" style="margin-bottom:12px">
            <h4>🚀 Features</h4>
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
      <!-- ─── ENCRYPT TAB ─── -->
      {:else if activeTab === 4}
        <div class="tab-content">
          <h3>🔐 Encrypt / Decrypt</h3>
          <p class="subtitle">
            AES-256-GCM encryption and decryption for individual files. Encrypted files are saved with a .aes256 extension.
          </p>

          <!-- Mode toggle -->
          <div class="enc-mode-row">
            <button class="btn mode-btn" class:active={encMode === 'encrypt'} onclick={() => { encMode = 'encrypt'; encResult = null; }}>
              🔒 Encrypt
            </button>
            <button class="btn mode-btn" class:active={encMode === 'decrypt'} onclick={() => { encMode = 'decrypt'; encResult = null; }}>
              🔓 Decrypt
            </button>
          </div>

          <!-- Selected file -->
          {#if encFilePath}
            <div class="source-item">
              <span class="src-icon">📄</span>
              <span class="src-path">{encFilePath}</span>
              <button class="btn-remove" onclick={() => { encFilePath = ""; encResult = null; }}>✕</button>
            </div>
          {/if}

          <!-- Password input -->
          <div class="enc-pw-row">
            <input
              type="password"
              class="enc-pw-input"
              placeholder="Enter {encMode === 'encrypt' ? 'encryption' : 'decryption'} password"
              bind:value={encPassword}
              disabled={busy}
            />
          </div>

          <!-- Action buttons -->
          <div class="btn-row">
            <button class="btn" onclick={browseEncFile} disabled={busy}>
              📄 Select File
            </button>
            {#if encFilePath}
              <button class="btn" onclick={clearEnc} disabled={busy}>
                🗑️ Clear
              </button>
            {/if}
          </div>

          <button class="btn-primary" onclick={doEncrypt} disabled={busy || !encFilePath || !encPassword}>
            {busy
              ? (encMode === 'encrypt' ? '🔄 Encrypting...' : '🔄 Decrypting...')
              : (encMode === 'encrypt' ? '🔐 Encrypt File' : '🔓 Decrypt File')}
          </button>

          <!-- Result -->
          {#if encResult}
            <div class="result-card success">
              <span class="result-icon">✅</span>
              <div>
                <strong>{encMode === 'encrypt' ? 'Encrypted' : 'Decrypted'} successfully</strong><br />
                <span class="muted">{encResult}</span>
              </div>
            </div>
          {/if}

          <!-- Drop zone -->
          {#if !encFilePath && !busy}
            <div class="dropzone" role="button" tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onDrop}>
              <span style="font-size:32px">🔐</span>
              <p>Drop a file here to {encMode}</p>
              <span class="hint">or click Select File to browse</span>
            </div>
          {/if}
        </div>
      {/if}
    </main>
  </div>

  <div class="statusbar">
    <div class="sb-left">
      <span class="status-dot" class:busy-dot={busy}></span>
      ZipLoom — Archive Utility
      {#if busy}<span style="margin-left:8px;color:var(--primary)">Processing...</span>{/if}
    </div>
    <div class="sb-center"></div>
    <div class="sb-right">
      <span class="offline-badge">🔒 Offline</span>
    </div>
  </div>
</div>

<style>
  .tab-content { max-width: 680px; margin: 0 auto; }
  .tab-content h3 { margin: 0 0 4px; font-size: 18px; }
  .subtitle { color: var(--text-secondary); font-size: 13px; margin: 0 0 20px; }

  /* Toast */
  .toast {
    position: fixed; top: 56px; left: 50%; transform: translateX(-50%);
    background: var(--card); border: 1px solid var(--border); border-radius: var(--radius);
    padding: 10px 24px; font-size: 13px; z-index: 100; box-shadow: 0 4px 16px rgba(0,0,0,0.4);
  }
  .toast.error { border-color: var(--danger); color: var(--danger); }

  /* Source list */
  .source-list { margin-bottom: 16px; }
  .source-list h4 { margin: 0 0 8px; font-size: 13px; color: var(--text-secondary); }
  .source-item {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 10px; background: var(--card-hover); border: 1px solid var(--border);
    border-radius: 6px; margin-bottom: 4px; font-size: 12px;
  }
  .src-icon { flex-shrink: 0; }
  .src-path { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-secondary); }
  .btn-remove { background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 14px; padding: 2px 6px; }
  .btn-remove:hover { color: var(--danger); }

  /* Format row */
  .format-row { display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }
  .format-row label { font-size: 13px; color: var(--text-secondary); }
  .format-row select {
    padding: 6px 12px; background: var(--card); border: 1px solid var(--border);
    border-radius: 6px; color: var(--text); font-size: 13px;
  }

  /* Buttons */
  .btn-row { display: flex; gap: 8px; margin-bottom: 16px; }
  .btn, .btn-primary {
    padding: 8px 18px; border-radius: 8px; font-size: 13px; border: 1px solid var(--border);
    background: var(--card); color: var(--text); cursor: pointer; transition: all 0.15s;
  }
  .btn:hover { border-color: var(--primary); }
  .btn:disabled, .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-primary { background: var(--primary); border-color: var(--primary); color: #fff; }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); }

  /* Result cards */
  .result-card {
    display: flex; align-items: flex-start; gap: 10px;
    margin-top: 16px; padding: 12px; border-radius: 8px; font-size: 13px;
  }
  .result-card.success { background: rgba(34,197,94,0.1); border: 1px solid var(--success); }
  .result-card.error { background: rgba(239,68,68,0.1); border: 1px solid var(--danger); }
  .result-card.info { background: rgba(59,130,246,0.1); border: 1px solid var(--primary); }
  .result-icon { font-size: 18px; flex-shrink: 0; }
  .muted { color: var(--text-muted); font-size: 11px; }

  /* Entry table */
  .entry-table { margin-top: 16px; border: 1px solid var(--border); border-radius: 8px; overflow: hidden; }
  .entry-header, .entry-row {
    display: grid; grid-template-columns: 1fr 100px 80px; gap: 12px;
    padding: 8px 12px; font-size: 12px; align-items: center;
  }
  .entry-header { background: var(--card-hover); font-weight: 600; color: var(--text-secondary); border-bottom: 1px solid var(--border); }
  .entry-row { border-bottom: 1px solid rgba(255,255,255,0.03); }
  .entry-row:last-child { border-bottom: none; }
  .entry-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text); }
  .entry-size { color: var(--text-secondary); text-align: right; }
  .entry-ratio { text-align: right; font-weight: 600; }
  .entry-ratio.good { color: var(--success); }
  .entry-ratio.bad { color: var(--warn); }

  /* Encrypt tab */
  .enc-mode-row { display: flex; gap: 8px; margin-bottom: 16px; }
  .mode-btn { flex: 1; justify-content: center; text-align: center; font-size: 14px; padding: 10px; }
  .mode-btn.active { background: var(--primary); border-color: var(--primary); color: #fff; }
  .enc-pw-row { margin-bottom: 16px; }
  .enc-pw-input {
    width: 100%; padding: 10px 14px; border-radius: 8px; border: 1px solid var(--border);
    background: var(--card); color: var(--text); font-size: 14px; box-sizing: border-box;
    outline: none; transition: border-color 0.15s;
  }
  .enc-pw-input:focus { border-color: var(--primary); }
  .enc-pw-input::placeholder { color: var(--text-muted); }
  .enc-pw-input:disabled { opacity: 0.5; }

  /* Dropzone */
  .dropzone {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    height: 180px; border: 2px dashed var(--border); border-radius: var(--radius-lg);
    background: var(--card-hover); cursor: pointer; gap: 8px; transition: border-color 0.2s;
    margin-top: 20px;
  }
  .dropzone:hover { border-color: var(--primary); }
  .dropzone p { margin: 0; font-size: 14px; color: var(--text-secondary); }
  .hint { font-size: 11px; color: var(--text-muted); }

  /* Status busy dot */
  .busy-dot { background: var(--primary) !important; animation: pulse 1s infinite; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.3; } }
</style>
