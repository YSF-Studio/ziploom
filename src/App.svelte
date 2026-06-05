<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

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
  let encMode = $state("encrypt");
  let encResult = $state(null);

  function timeoutPromise(promise, ms) {
    let timer;
    const timeout = new Promise((_, reject) => {
      timer = setTimeout(() => reject("TIMEOUT"), ms);
    });
    return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
  }

  const tabs = [
    { label: "Compress", icon: "\u{1F4E6}" },
    { label: "Extract", icon: "\u{1F4C2}" },
    { label: "Inspect", icon: "\u{1F50D}" },
    { label: "About", icon: "\u2139\uFE0F" },
    { label: "Encrypt", icon: "\u{1F512}" },
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

  // ─── Tauri drag-drop listener ───
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
    <span class="title">ZipLoom</span>
    <span class="title-sep">—</span>
    <span class="title-desc">Archive Utility</span>
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

      <!-- ─── COMPRESS TAB ─── (redesigned like screenshot) -->
      {#if activeTab === 0}
        <div class="tab-content compress-tab">
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

            <div class="format-row">
              <label for="format-select">Format:</label>
              <select id="format-select" bind:value={compFormat} disabled={busy}>
                {#each formats as f}
                  <option value={f.value}>{f.label}</option>
                {/each}
              </select>
            </div>

            <button class="btn-primary" onclick={doCompress} disabled={busy || compSources.length === 0}>
              {busy ? '🔄 Compressing...' : '🗜️ Compress'}
            </button>

            {#if compResult}
              <div class="result-card success">
                <span class="result-icon">✅</span>
                <div>
                  <strong>{compResult.message}</strong><br />
                  <span class="muted">{compResult.filesProcessed} files → {compResult.outputPath}</span>
                </div>
              </div>
            {/if}
          {:else if !busy}
            <div class="dropzone-lg" role="button" tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onDrop}
              onclick={() => browseSources()}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  browseSources();
                }
              }}>
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="color:var(--text-muted);margin-bottom:8px">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="7 10 12 15 17 10"/>
                <line x1="12" y1="15" x2="12" y2="3"/>
              </svg>
              <p class="dz-title">Drop files here or click to browse</p>
              <p class="dz-hint">Tip: Click 'Browse Folder' below for entire directories.</p>
            </div>
          {/if}

          <div class="compress-actions">
            <button class="btn action-btn" onclick={browseSources} disabled={busy}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
              </svg>
              Browse Files
            </button>
            <button class="btn action-btn" onclick={browseDir} disabled={busy}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              Browse Folder
            </button>
          </div>
        </div>

      <!-- ─── EXTRACT TAB ─── (redesigned like screenshot) -->
      {:else if activeTab === 1}
        <div class="tab-content extract-tab">
          {#if extrArchive}
            <div class="source-item" style="margin-bottom:12px">
              <span class="src-icon">📦</span>
              <span class="src-path">{extrArchive}</span>
            </div>

            <button class="btn-primary" onclick={browseArchive} disabled={busy}>
              {busy ? '🔄 Extracting...' : '📂 Choose Archive & Extract'}
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
          {:else if !busy}
            <div class="dropzone-lg" role="button" tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onDrop}
              onclick={() => browseArchive()}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  browseArchive();
                }
              }}>
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="color:var(--text-muted);margin-bottom:8px">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="7 10 12 15 17 10"/>
                <line x1="12" y1="15" x2="12" y2="3"/>
                <path d="M12 3v12"/>
              </svg>
              <p class="dz-title">Drop archive here or click to browse</p>
            </div>
          {/if}
        </div>

      <!-- ─── INSPECT TAB ─── (redesigned like screenshot) -->
      {:else if activeTab === 2}
        <div class="tab-content inspect-tab">
          {#if inspArchive}
            <div class="source-item" style="margin-bottom:12px">
              <span class="src-icon">🗄️</span>
              <span class="src-path">{inspArchive}</span>
            </div>

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
          {:else if !busy}
            <div class="dropzone-lg" role="button" tabindex="0"
              ondragover={(e) => e.preventDefault()}
              ondrop={onDrop}
              onclick={() => browseInspect()}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  browseInspect();
                }
              }}>
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="color:var(--text-muted);margin-bottom:8px">
                <circle cx="11" cy="11" r="8"/>
                <line x1="21" y1="21" x2="16.65" y2="16.65"/>
              </svg>
              <p class="dz-title">Drop archive here for forensic inspection</p>
              <p class="dz-hint">Scan archives for malware, verify integrity, inspect file contents</p>
            </div>

            <div class="inspect-links">
              <button class="inspect-link" onclick={browseInspect} disabled={busy}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
                  <line x1="8" y1="11" x2="14" y2="11"/>
                </svg>
                Full Scan — Detect threats &amp; anomalies
              </button>
              <button class="inspect-link" onclick={browseInspect} disabled={busy}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>
                </svg>
                Hash All — MD5, SHA1, SHA256 checksums
              </button>
              <button class="inspect-link" onclick={browseInspect} disabled={busy}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
                </svg>
                Export CSV — Forensic report
              </button>
              <button class="inspect-link" onclick={browseInspect} disabled={busy}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                </svg>
                Extract Selected — Pick specific files
              </button>
            </div>

            <div class="inspect-browse">
              <button class="btn action-btn" onclick={browseInspect} disabled={busy}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                Browse Archive
              </button>
            </div>
          {/if}
        </div>

      <!-- ─── ABOUT TAB ─── (redesigned to match screenshot) -->
      {:else if activeTab === 3}
        <div class="about-page">
          <div class="about-hero">
            <div class="about-icon">
              <svg width="72" height="72" viewBox="0 0 72 72" fill="none">
                <rect width="72" height="72" rx="16" fill="#3B82F6"/>
                <rect x="14" y="20" width="44" height="34" rx="4" fill="white" opacity="0.95"/>
                <rect x="20" y="28" width="32" height="4" rx="2" fill="#3B82F6" opacity="0.3"/>
                <rect x="20" y="36" width="24" height="4" rx="2" fill="#3B82F6" opacity="0.3"/>
                <rect x="20" y="44" width="16" height="4" rx="2" fill="#3B82F6" opacity="0.3"/>
                <rect x="30" y="14" width="4" height="14" rx="2" fill="white"/>
                <rect x="36" y="10" width="4" height="18" rx="2" fill="white"/>
              </svg>
            </div>
            <h1 class="about-title">ZipLoom</h1>
            <p class="about-version">Version 1.0</p>
            <p class="about-tagline">Archive Utility &amp; Forensic Inspector</p>
            <p class="about-subtext">Pure Rust · Offline · Private</p>
          </div>

          <div class="about-features">
            {#each [
              '8 formats: ZIP · TAR · GZ · BZ2 · XZ · Zstandard — compress & extract',
              '7-Zip & RAR — extract support',
              'AES-256 encrypted archives (ZIP)',
              'Zstandard (.zst/.tzst) — modern fast compression',
              'Forensic analysis — magic byte verification',
              'Entropy scanning & anomaly detection',
              'Batch hashing (MD5, SHA-1, SHA-256)',
              'Archive conversion & split volumes',
              '100% offline · zero data collection',
            ] as feat}
              <div class="feature-row">
                <span class="feature-check">✅</span>
                <span class="feature-text">{feat}</span>
              </div>
            {/each}
          </div>

          <div class="about-disclaimer">
            <div class="disclaimer-header">⚠️ <strong>LEGAL DISCLAIMER</strong></div>
            <p>
              ZipLoom is provided "AS-IS" with NO WARRANTY of any kind, express or implied.
              NOT certified for court evidence, ISO auditing, NIST validation, or any forensic standard compliance.
              All forensic analysis results are informational only. Users must independently verify all findings before
              use in any legal, compliance, or security context.
              This tool is intended for authorized security research and personal archive management only.
            </p>
          </div>

          <div class="about-footer">
            <p class="footer-hearts">Made with ❤️ by Yusuf Shalahuddin Al Ayyubi As Sobari</p>
            <p class="footer-brand">YSF Studio</p>
          </div>
        </div>

      <!-- ─── ENCRYPT TAB ─── -->
      {:else if activeTab === 4}
        <div class="tab-content">
          <h3>🔐 Encrypt / Decrypt</h3>
          <p class="subtitle">
            AES-256-GCM encryption and decryption for individual files. Encrypted files are saved with a .aes256 extension.
          </p>

          <div class="enc-mode-row">
            <button class="btn mode-btn" class:active={encMode === 'encrypt'} onclick={() => { encMode = 'encrypt'; encResult = null; }}>
              🔒 Encrypt
            </button>
            <button class="btn mode-btn" class:active={encMode === 'decrypt'} onclick={() => { encMode = 'decrypt'; encResult = null; }}>
              🔓 Decrypt
            </button>
          </div>

          {#if encFilePath}
            <div class="source-item">
              <span class="src-icon">📄</span>
              <span class="src-path">{encFilePath}</span>
              <button class="btn-remove" onclick={() => { encFilePath = ""; encResult = null; }}>✕</button>
            </div>
          {/if}

          <div class="enc-pw-row">
            <input
              type="password"
              class="enc-pw-input"
              placeholder="Enter {encMode === 'encrypt' ? 'encryption' : 'decryption'} password"
              bind:value={encPassword}
              disabled={busy}
            />
          </div>

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

          {#if encResult}
            <div class="result-card success">
              <span class="result-icon">✅</span>
              <div>
                <strong>{encMode === 'encrypt' ? 'Encrypted' : 'Decrypted'} successfully</strong><br />
                <span class="muted">{encResult}</span>
              </div>
            </div>
          {/if}

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

  /* Titlebar */
  .titlebar .title-sep { color: var(--text-muted); font-size: 13px; margin: 0 4px; }
  .titlebar .title-desc { color: var(--text-muted); font-size: 13px; }

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

  /* Busy dot */
  .busy-dot { background: var(--primary) !important; animation: pulse 1s infinite; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.3; } }

  /* ─── ABOUT PAGE (redesigned like screenshot) ─── */
  .about-page {
    max-width: 580px;
    margin: 0 auto;
    padding: 20px 0;
  }

  .about-hero {
    text-align: center;
    margin-bottom: 28px;
  }

  .about-icon {
    display: flex;
    justify-content: center;
    margin-bottom: 12px;
  }

  .about-title {
    margin: 0 0 2px;
    font-size: 28px;
    font-weight: 700;
  }

  .about-version {
    margin: 0 0 6px;
    font-size: 13px;
    color: var(--text-muted);
  }

  .about-tagline {
    margin: 0 0 2px;
    font-size: 14px;
    color: var(--text-secondary);
  }

  .about-subtext {
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  .about-features {
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .feature-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .feature-check {
    font-size: 14px;
    flex-shrink: 0;
  }

  .feature-text {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .about-disclaimer {
    margin-bottom: 20px;
    padding: 12px 14px;
    border: 1px solid rgba(234,179,8,0.3);
    border-radius: 8px;
    background: rgba(234,179,8,0.06);
  }

  .disclaimer-header {
    font-size: 13px;
    margin-bottom: 8px;
    color: var(--warn);
  }

  .about-disclaimer p {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .about-footer {
    text-align: center;
    padding-top: 4px;
  }

  .footer-hearts {
    margin: 0 0 4px;
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
  }

  .footer-brand {
    margin: 0;
    font-size: 12px;
    color: var(--primary);
  }

  /* ─── Shared: Dropzone Large ─── */
  .dropzone-lg {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 220px;
    border: 2px dashed var(--border);
    border-radius: var(--radius-lg);
    background: var(--card-hover);
    cursor: pointer;
    gap: 4px;
    transition: border-color 0.2s, background 0.2s;
  }
  .dropzone-lg:hover {
    border-color: var(--primary);
    background: color-mix(in srgb, var(--card-hover) 90%, var(--primary) 10%);
  }
  .dz-title {
    margin: 0;
    font-size: 15px;
    font-weight: 500;
    color: var(--text-secondary);
  }
  .dz-hint {
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  /* ─── Compress Tab ─── */
  .compress-tab .compress-actions {
    display: flex;
    gap: 10px;
    justify-content: center;
    margin-top: 16px;
  }
  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 10px 22px;
    border-radius: 8px;
    font-size: 13px;
    border: 1px solid var(--border);
    background: var(--card);
    color: var(--text);
    cursor: pointer;
    transition: all 0.15s;
  }
  .action-btn:hover { border-color: var(--primary); }
  .action-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  /* ─── Inspect Tab ─── */
  .inspect-links {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: center;
    margin-top: 16px;
  }
  .inspect-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 12px;
    border: 1px solid var(--border);
    background: var(--card);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
    text-align: left;
  }
  .inspect-link:hover {
    border-color: var(--primary);
    color: var(--text);
  }
  .inspect-link:disabled { opacity: 0.4; cursor: not-allowed; }
  .inspect-link svg { flex-shrink: 0; }

  .inspect-browse {
    display: flex;
    justify-content: center;
    margin-top: 16px;
  }
</style>
