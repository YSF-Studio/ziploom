<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { writeTextFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";

  let tab = $state(0);
  let formats = $state([]);
  let format = $state("zip");
  let files = $state([]);
  let archive = $state(null);
  let dest = $state("");
  let password = $state("");
  let usePW = $state(false);
  let cleanMeta = $state(true);
  let compressLevel = $state(6);
  let busy = $state(false);
  let progress = $state(0);
  let status = $state("");
  let msg = $state("");
  let archiveName = $state("");
  let errMsg = $state("");

  // Compress extras
  let splitEnabled = $state(false);
  let splitSize = $state("10");

  // Compress result (shown after compress)
  let compressResult = $state(null);

  // Extract
  let pendingPw = $state("");
  let pendingSrc = $state("");
  let pendingDst = $state("");

  // Inspect
  let insArchive = $state("");
  let insPassword = $state("");
  let insBusy = $state(false);
  let insEntries = $state([]);
  let insAnomalies = $state([]);
  let insReport = $state(null);
  let insViewMode = $state("tree");
  let insSortBy = $state("name");
  let insSelected = $state([]);
  let insHashes = $state([]);
  let insPreviewContent = $state("");
  let insPreviewName = $state("");

  // Inspect column toggles
  let colHashes = $state(false);
  let colEntropy = $state(true);
  let colMagic = $state(true);
  let colTime = $state(false);
  let colSelect = $state(true);

  onMount(async () => {
    formats = await invoke("get_formats");
    await listen("progress", (e) => { progress = e.payload.percent; status = e.payload.status; });
    document.addEventListener('contextmenu', (e) => e.preventDefault());
  });

  function onDrop(e) {
    e.preventDefault();
    const paths = [...e.dataTransfer.files].map(f => f.path);
    if (tab === 0) files = [...files, ...paths];
    else if (tab === 1 && paths.length > 0) { archiveName = paths[0]; detect(paths[0]); }
    else if (tab === 2 && paths.length > 0) { insArchive = paths[0]; insPreviewContent = ""; insPreviewName = ""; insSelected = []; msg = ""; insPassword = ""; loadForensic(paths[0]); }
  }

  async function detect(p) {
    const f = await invoke("detect_format_cmd", { path: p });
    archive = f || null;
  }

  async function pickFiles() {
    const sel = await open({ multiple: true, title: "Select files or folders" });
    if (sel) files = [...files, ...sel];
  }

  async function pickFolder() {
    const sel = await open({ directory: true, title: "Select folder to compress" });
    if (sel) files = [...files, sel];
  }

  async function pickArchive() {
    const sel = await open({ multiple: false, title: "Select archive" });
    if (sel) { archiveName = sel; detect(sel); }
  }

  async function pickInsArchive() {
    const sel = await open({ multiple: false, title: "Select archive to inspect" });
    if (sel) { insArchive = sel; loadForensic(sel); }
  }

  function removeFile(idx) { files = files.filter((_, i) => i !== idx); }

  // ─── COMPRESS ───
  async function doCompress() {
    if (files.length === 0) return;
    const ext = formats.find(f => f.id === format)?.ext[0] || "zip";
    const dst = await save({ defaultPath: `Archive.${ext}`, title: "Save Archive" });
    if (!dst) return;
    compressResult = null;
    busy = true; progress = 0; status = "Compressing..."; msg = "";
    try {
      const result = await invoke("compress", {
        args: {
          sources: files, destination: dst, format,
          password: usePW ? password : null, cleanMeta, level: compressLevel,
          splitSize: splitEnabled ? parseInt(splitSize) : 0,
          checksumAlgo: "auto"
        }
      });
      msg = `✅ Saved to ${result}`; status = "Done!";
      // Show result summary
      const name = result.split(/[/\\]/).pop();
      let summary = `📁 **${name}**\n📂 Location: ${result}`;
      if (usePW && password) summary += `\n🔐 Password: ${password} (AES-256)`;
      summary += `\n🧹 Clean metadata: ${cleanMeta ? "Yes" : "No"}`;
      summary += `\n📊 Compression level: ${compressLevel}/9`;
      // Split hidden — available via Tools menu
      // Auto compute hash
      try {
        const h = await invoke("checksum", { path: result, algorithm: "md5" });
        const s1 = await invoke("checksum", { path: result, algorithm: "sha1" });
        const s256 = await invoke("checksum", { path: result, algorithm: "sha256" });
        summary += `\n🔐 MD5: ${h}\n🔐 SHA1: ${s1}\n🔐 SHA256: ${s256}`;
      } catch(_) {}
      compressResult = summary;
    } catch (e) { msg = `❌ ${e}`; }
    busy = false; progress = 1;
  }

  // ─── EXTRACT ───
  async function doExtract() {
    if (!archiveName) return;
    const d = dest || (await open({ directory: true, title: "Select destination" }));
    if (!d) return;
    busy = true; progress = 0; status = "Extracting..."; msg = ""; errMsg = "";
    try {
      const r = await invoke("extract", {
        args: { source: archiveName, destination: d, password: usePW ? password : null, cleanMeta }
      });
      msg = `✅ Extracted to ${r}`; status = "Done!";
    } catch (e) {
      if (e === "PASSWORD_NEEDED") { pendingSrc = archiveName; pendingDst = d; showPW = true; }
      else { msg = `❌ ${e}`; }
    }
    busy = false;
  }

  let showPW = $state(false);
  async function extractWithPW() {
    showPW = false; busy = true; status = "Extracting..."; msg = "";
    try {
      const r = await invoke("extract", {
        args: { source: pendingSrc, destination: pendingDst, password: pendingPw, cleanMeta }
      });
      msg = `✅ Extracted to ${r}`; status = "Done!";
    } catch (e) {
      if (e === "PASSWORD_NEEDED") { errMsg = "Wrong password!"; showPW = true; }
      else { msg = `❌ ${e}`; }
    }
    busy = false; progress = 1;
  }

  // ─── INSPECT ───
  async function loadForensic(src) {
    if (!src) return;
    // Clear ALL previous state
    insBusy = true; insEntries = []; insAnomalies = []; insReport = null; insHashes = [];
    insPreviewContent = ""; insPreviewName = ""; insSelected = []; msg = ""; insPassword = "";
    try {
      const entries = await invoke("forensic_load", {
        source: src, password: insPassword || null
      });
      insEntries = entries;
    } catch (e) {
      if (e === "PASSWORD_NEEDED") { forensicPwOpen = true; }
      else { msg = `❌ ${e}`; }
    }
    insBusy = false;
  }

  let forensicPwOpen = $state(false);
  let forensicPw = $state("");

  async function submitForensicPw() {
    forensicPwOpen = false;
    insPassword = forensicPw;
    await loadForensic(insArchive);
  }

  async function doBatchHash() {
    if (!insArchive) return;
    insBusy = true;
    try {
      insHashes = await invoke("batch_hash", { path: insArchive, password: insPassword || null });
    } catch (e) { msg = `❌ ${e}`; }
    insBusy = false;
  }

  async function doFullReport() {
    if (!insArchive) return;
    insBusy = true;
    try {
      const report = await invoke("generate_forensic_report", { path: insArchive, password: insPassword || null });
      insReport = report;
      insEntries = report.entries;
      insAnomalies = report.anomalies;
    } catch (e) {
      if (e === "PASSWORD_NEEDED") { forensicPwOpen = true; }
      else { msg = `❌ ${e}`; }
    }
    insBusy = false;
  }

  async function exportReportCSV() {
    if (!insReport && insEntries.length === 0) return;
    const dst = await save({ defaultPath: "forensic_report.csv", title: "Save Report CSV" });
    if (!dst) return;
    let csv = "⚠️ LEGAL DISCLAIMER: This report is generated by ZipLoom forensic analysis tool.\n";
    csv += "⚠️ ZipLoom is NOT certified for court evidence, ISO auditing, or NIST validation.\n";
    csv += "⚠️ All results are informational only. Verify independently before legal use.\n";
    csv += "⚠️ Generated by ZipLoom v1.0 (YSF Studio) — https://ysfloom.com\n";
    csv += "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n";
    csv += "Path,Size,MD5,SHA1,SHA256,Entropy,MagicMatch,Detected,Expected\n";
    const data = insReport ? insReport.entries : insEntries;
    for (const e of data) {
      csv += `"${e.path}",${e.size},${e.md5||""},${e.sha1||""},${e.sha256||""},${e.entropy||""},${e.magic_match||""},${e.detected_type||""},${e.expected_type||""}\n`;
    }
    try {
      await writeTextFile(dst, csv);
      msg = `✅ Report saved to ${dst}`;
    } catch (e) { msg = `❌ ${e}`; }
  }

  async function doSelectiveExtract() {
    if (!insArchive || insSelected.length === 0) return;
    const dst = await open({ directory: true, title: "Extract selected files to" });
    if (!dst) return;
    insBusy = true;
    try {
      const result = await invoke("selective_extract", {
        source: insArchive, password: insPassword || null, files: insSelected, destination: dst
      });
      msg = result;
    } catch (e) { msg = `❌ ${e}`; }
    insBusy = false;
  }

  function toggleSelect(path) {
    insSelected = insSelected.includes(path) ? insSelected.filter(p => p !== path) : [...insSelected, path];
  }

  function toggleSelectAll() {
    insSelected = insSelected.length === insEntries.length ? [] : insEntries.map(e => e.path);
  }

  function previewFile(entry) {
    insPreviewName = entry.path;
    let c = `📄 ${entry.path}\n${'━'.repeat(40)}\n`;
    c += `Size: ${formatSize(entry.size)} (${entry.size} bytes)\n`;
    if (entry.compressed_size) c += `Compressed: ${formatSize(entry.compressed_size)}\n`;
    if (entry.ratio != null) c += `Ratio: ${(entry.ratio * 100).toFixed(1)}%\n`;
    if (entry.modified) c += `Modified: ${entry.modified}\n`;
    if (entry.permissions) c += `Permissions: ${entry.permissions}\n`;
    if (entry.md5) c += `\nMD5:   ${entry.md5}\n`;
    if (entry.sha1) c += `SHA1:  ${entry.sha1}\n`;
    if (entry.sha256) c += `SHA256:${entry.sha256}\n`;
    if (entry.entropy != null) {
      c += `\nEntropy: ${entry.entropy.toFixed(4)} / 8.0`;
      if (entry.entropy > 8.0) c += ` ⚠️ HIGH — Possible encrypted/suspicious`;
      else if (entry.entropy > 7.0) c += ` ⚡ Elevated — Possibly compressed data`;
    }
    c += `\n\n📖 What is Entropy?`;
    c += `\nEntropy measures how random data looks.`;
    c += `\n• 0.0–4.0: Text, structured data (normal)`;
    c += `\n• 4.0–7.0: Compressed or mixed data`;
    c += `\n• 7.0–8.0: Encrypted or high-compression content`;
    c += `\n• > 8.0: ⚠️ Flagged — review this file!\n`;
    if (entry.magic_match != null) {
      c += `\nMagic Bytes: ${entry.magic_match ? '✅ Match' : '❌ MISMATCH!'}`;
      if (entry.detected_type) c += `\nDetected type: ${entry.detected_type}`;
      if (entry.expected_type) c += `\nExpected (by extension): ${entry.expected_type}`;
      c += `\n\n📖 What are Magic Bytes?`;
      c += `\nMagic bytes are the first few bytes of a file that`;
      c += `\nidentify its actual type, regardless of extension.`;
      c += `\nIf a .jpg has ZIP magic bytes, it's suspicious!`;
    }
    insPreviewContent = c;
  }

  // Auto-dismiss toast
  $effect(() => {
    if (msg) { const t = setTimeout(() => msg = "", 6000); return () => clearTimeout(t); }
  });

  const LEVEL_NAMES = ["Store","Fastest","Fast","Fast","Normal","Normal","Good","Good","Maximum","Maximum"];
  function compressLabel(l) { return LEVEL_NAMES[l] || `Level ${l}`; }

  function sortedEntries() {
    let list = [...insEntries];
    if (insSortBy === "size") list.sort((a, b) => b.size - a.size);
    else if (insSortBy === "date") list.sort((a, b) => (b.modified||"") < (a.modified||"") ? -1 : 1);
    else list.sort((a, b) => a.path.localeCompare(b.path));
    return list;
  }

  // Build tree from flat entries
  function buildTree(entries) {
    const root = { name: "", children: [], isDir: true, expanded: true };
    for (const e of entries) {
      if (e.is_dir) continue;
      const parts = e.path.split("/");
      let node = root;
      for (let i = 0; i < parts.length - 1; i++) {
        let child = node.children.find(c => c.name === parts[i] && c.isDir);
        if (!child) {
          child = { name: parts[i], children: [], isDir: true, expanded: false };
          node.children.push(child);
        }
        node = child;
        if (!node.expanded) break;
      }
      node.children.push({ name: parts[parts.length-1], isDir: false, entry: e });
    }
    return root.children;
  }

  let treeExpanded = $state({});
  function toggleTree(name) {
    treeExpanded[name] = !treeExpanded[name];
  }

  function formatSize(s) {
    if (s >= 1073741824) return (s / 1073741824).toFixed(1) + " GB";
    if (s >= 1048576) return (s / 1048576).toFixed(1) + " MB";
    if (s >= 1024) return (s / 1024).toFixed(1) + " KB";
    return s + " B";
  }
</script>

<main class="app" ondragover={(e) => e.preventDefault()} ondrop={onDrop}>
  <header class="header">
    <h1>ZipLoom</h1>
    <nav class="tabs">
      <button class="tab" class:active={tab===0} onclick={() => tab=0}>📦 Compress</button>
      <button class="tab" class:active={tab===1} onclick={() => tab=1}>📂 Extract</button>
      <button class="tab" class:active={tab===2} onclick={() => tab=2}>🔍 Inspect</button>
      <button class="tab" class:active={tab===3} onclick={() => tab=3}>ℹ️ About</button>
    </nav>
  </header>

  <!-- ═══ COMPRESS ═══ -->
  {#if tab === 0}
    <section class="page">
      <div class="dropzone" ondrop={onDrop} onclick={pickFiles} onkeydown={(e) => e.key === 'Enter' && pickFiles()} role="button" tabindex="0">
        <div class="drop-icon">📥</div>
        <p>Drop files here or click to browse</p>
        <p class="hint">Tip: Click <strong>"Browse Folder"</strong> below for entire directories</p>
      </div>

      {#if files.length > 0}
        <div class="file-list">
          {#each files as f, i}
            <div class="file-row">
              <span>📄 {f.split(/[/\\]/).pop()}</span>
              <button class="btn-small" onclick={() => removeFile(i)}>✕</button>
            </div>
          {/each}
        </div>

        <div class="btn-row">
          <button class="btn-secondary" onclick={pickFiles}>📂 Browse Files</button>
          <button class="btn-secondary" onclick={pickFolder}>📁 Browse Folder</button>
        </div>

        <div class="options">
          <label>Format:
            <select bind:value={format}>
              {#each formats.filter(f => f.compress) as fmt}
                <option value={fmt.id}>{fmt.name} — {fmt.desc}</option>
              {/each}
            </select>
          </label>

          <label><input type="checkbox" bind:checked={usePW} /> Password protect</label>
          {#if usePW}
            <input type="password" bind:value={password} placeholder="Enter password" class="input" />
          {/if}

          <label><input type="checkbox" bind:checked={cleanMeta} /> Clean Mac metadata (__MACOSX/, .DS_Store)</label>

          <div class="level-row">
            <span class="level-label">Compression: {compressLabel(compressLevel)}</span>
            <input type="range" min="0" max="9" bind:value={compressLevel} class="level-slider" />
            <span class="level-hint">{compressLabel(compressLevel)}</span>
          </div>

          {#if usePW && format === 'zip'}
            <p class="badge">🔒 AES-256 encryption</p>
          {/if}

          <!-- Split volumes available via split_archive command (Tools menu) -->
        </div>

        <button class="btn-primary" onclick={doCompress} disabled={busy}>
          {busy ? "⏳ Compressing..." : "📦 Compress"}
        </button>

        {#if compressResult}
          <div class="result-card">
            <pre class="result-text">{compressResult}</pre>
          </div>
        {/if}
      {:else}
        <div class="btn-row" style="margin-top:-8px">
          <button class="btn-secondary" onclick={pickFiles}>📂 Browse Files</button>
          <button class="btn-secondary" onclick={pickFolder}>📁 Browse Folder</button>
        </div>
      {/if}
    </section>

  <!-- ═══ EXTRACT ═══ -->
  {:else if tab === 1}
    <section class="page">
      <div class="dropzone" ondrop={onDrop} onclick={pickArchive} onkeydown={(e) => e.key === 'Enter' && pickArchive()} role="button" tabindex="0">
        <div class="drop-icon">📂</div>
        {#if archiveName}
          <p>{archiveName.split(/[/\\]/).pop()}
            <button class="clear-btn" onclick={(e) => { e.stopPropagation(); archiveName=""; archive=null; }}>✕</button>
          </p>
          {#if archive}
            <p class="badge">{archive.name} archive</p>
          {:else}
            <p class="err">Unknown format</p>
          {/if}
        {:else}
          <p>Drop archive here or click to browse</p>
        {/if}
      </div>

      {#if archiveName}
        <div class="options">
          <label><input type="checkbox" bind:checked={cleanMeta} /> Remove __MACOSX/ and .DS_Store</label>
        </div>

        <button class="btn-primary" onclick={doExtract} disabled={busy}>
          {busy ? "⏳ Extracting..." : "📂 Extract"}
        </button>
      {/if}
    </section>

  <!-- ═══ INSPECT ═══ -->
  {:else if tab === 2}
    <section class="page">
      <div class="dropzone" ondrop={onDrop} onclick={pickInsArchive} onkeydown={(e) => e.key === 'Enter' && pickInsArchive()} role="button" tabindex="0">
        <div class="drop-icon">🔍</div>
        {#if insArchive}
          <p>{insArchive.split(/[/\\]/).pop()}
            <button class="clear-btn" onclick={(e) => { e.stopPropagation(); insArchive=""; insEntries=[]; insAnomalies=[]; insReport=null; insHashes=[]; insPreviewContent=""; insPreviewName=""; insSelected=[]; msg=""; insPassword=""; }}>✕</button>
          </p>
        {:else}
          <p>Drop archive here for forensic inspection</p>
        {/if}
      </div>

      {#if insArchive}
        <div class="ins-toolbar">
          <button class="btn-secondary" onclick={doFullReport} disabled={insBusy}>
            {insBusy ? "⏳" : "🔬 Full Scan"}
          </button>
          <button class="btn-secondary" onclick={doBatchHash} disabled={insBusy}>🔐 Hash All</button>
          <button class="btn-secondary" onclick={exportReportCSV} disabled={insEntries.length === 0}>📊 Export CSV</button>
        </div>

        {#if insEntries.length > 0}
          <!-- Controls: view mode, sort, column toggles -->
          <div class="ins-controls">
            <button class="btn-small-ins" class:active={insViewMode==='tree'} onclick={() => insViewMode='tree'}>🌲 Tree</button>
            <button class="btn-small-ins" class:active={insViewMode==='flat'} onclick={() => insViewMode='flat'}>📋 Flat</button>
            <select bind:value={insSortBy} class="input" style="width:auto;display:inline;margin:0 8px">
              <option value="name">Sort: Name</option>
              <option value="size">Sort: Size</option>
              <option value="date">Sort: Date</option>
            </select>
            <span class="file-count">{insEntries.length} files</span>
          </div>

          <!-- Column visibility toggles -->
          <div class="col-toggles">
            <label><input type="checkbox" bind:checked={colHashes} /> MD5/SHA</label>
            <label><input type="checkbox" bind:checked={colEntropy} /> Entropy</label>
            <label><input type="checkbox" bind:checked={colMagic} /> Magic</label>
            <label><input type="checkbox" bind:checked={colTime} /> Timestamp</label>
          </div>

          <!-- Anomalies -->
          {#if insAnomalies.length > 0}
            <div class="anomalies">
              <h4>⚠️ {insAnomalies.length} Anomalies Detected</h4>
              {#each insAnomalies as a}
                <p class="anomaly-item">{a.severity === "high" ? "🔴" : "🟡"} <strong>{a.file}</strong>: {a.issue}</p>
              {/each}
            </div>
          {/if}

          <!-- File Table -->
          <div class="ins-table-wrap">
            <div class="ins-table" style="min-width:{(colHashes?350:200)+(colEntropy?80:0)+(colMagic?70:0)+(colTime?170:0)}px">
              <div class="ins-header">
                {#if !colSelect}{:else}<label><input type="checkbox" checked={insSelected.length === insEntries.length && insEntries.length > 0} onchange={toggleSelectAll} /></label>{/if}
                <span class="col-name">Name</span>
                <span class="col-size">Size</span>
                {#if colHashes}<span class="col-hash">MD5</span>
                <span class="col-hash">SHA256</span>{/if}
                {#if colEntropy}<span class="col-entropy">Entropy</span>{/if}
                {#if colMagic}<span class="col-magic">Magic</span>{/if}
                {#if colTime}<span class="col-time">Modified</span>{/if}
              </div>
              <div class="ins-body">
                {#if insViewMode === 'flat'}
                  {#each sortedEntries() as entry}
                    <div class="ins-row" class:selected={insSelected.includes(entry.path)}>
                      {#if !colSelect}{:else}<label><input type="checkbox" checked={insSelected.includes(entry.path)} onchange={() => toggleSelect(entry.path)} /></label>{/if}
                      <span class="col-name" onclick={() => previewFile(entry)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && previewFile(entry)}>
                        {entry.is_dir ? "📁 " : "📄 "}{entry.path.split("/").pop() || entry.path}
                      </span>
                      <span class="col-size">{formatSize(entry.size)}</span>
                      {#if colHashes}<span class="col-hash hash-short">{(entry.md5||"").slice(0,8)}</span>
                      <span class="col-hash hash-short">{(entry.sha256||"").slice(0,8)}</span>{/if}
                      {#if colEntropy}
                        <span class="col-entropy" title="Shannon entropy score (0-8). Higher = more random/suspicious.">
                          {entry.entropy != null ? entry.entropy.toFixed(2) : "—"}
                          {#if entry.entropy != null && entry.entropy > 8.0}
                            <span class="flag-high">⚠️</span>
                          {:else if entry.entropy != null && entry.entropy > 7.0}
                            <span class="flag-mid">⚡</span>
                          {/if}
                        </span>
                      {/if}
                      {#if colMagic}
                        <span class="col-magic" title="Magic bytes verification. ❌ = extension mismatch!">
                          {entry.magic_match === true ? "✅" : entry.magic_match === false ? "❌" : "—"}
                        </span>
                      {/if}
                      {#if colTime}<span class="col-time">{(entry.modified||"").slice(0,10)}</span>{/if}
                    </div>
                  {/each}
                {:else}
                  <!-- Tree view -->
                  {#each buildTree(sortedEntries().filter(e => !e.is_dir)) as node}
                    <div class="tree-row">
                      <span class="col-name" style="padding-left:0">📁 {node.name}/</span>
                      <span class="col-size">{node.children.filter(c => !c.isDir).length} files</span>
                      {#if colHashes}<span class="col-hash"></span><span class="col-hash"></span>{/if}
                      {#if colEntropy}<span class="col-entropy"></span>{/if}
                      {#if colMagic}<span class="col-magic"></span>{/if}
                      {#if colTime}<span class="col-time"></span>{/if}
                    </div>
                    {#each node.children.filter(c => !c.isDir) as child}
                      <div class="ins-row" class:selected={insSelected.includes(child.entry.path)} style="padding-left:20px">
                        {#if !colSelect}{:else}<label><input type="checkbox" checked={insSelected.includes(child.entry.path)} onchange={() => toggleSelect(child.entry.path)} /></label>{/if}
                        <span class="col-name" onclick={() => previewFile(child.entry)} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && previewFile(child.entry)}>
                          📄 {child.name}
                        </span>
                        <span class="col-size">{formatSize(child.entry.size)}</span>
                        {#if colHashes}<span class="col-hash hash-short">{(child.entry.md5||"").slice(0,8)}</span>
                        <span class="col-hash hash-short">{(child.entry.sha256||"").slice(0,8)}</span>{/if}
                        {#if colEntropy}
                          <span class="col-entropy">
                            {child.entry.entropy != null ? child.entry.entropy.toFixed(2) : "—"}
                            {#if child.entry.entropy != null && child.entry.entropy > 8.0}
                              <span class="flag-high">⚠️</span>
                            {:else if child.entry.entropy != null && child.entry.entropy > 7.0}
                              <span class="flag-mid">⚡</span>
                            {/if}
                          </span>
                        {/if}
                        {#if colMagic}
                          <span class="col-magic">{child.entry.magic_match === true ? "✅" : child.entry.magic_match === false ? "❌" : "—"}</span>
                        {/if}
                        {#if colTime}<span class="col-time">{(child.entry.modified||"").slice(0,10)}</span>{/if}
                      </div>
                    {/each}
                  {/each}
                {/if}
              </div>
            </div>
          </div>

          <!-- Actions -->
          <div class="ins-actions">
            <button class="btn-primary" onclick={doSelectiveExtract} disabled={insSelected.length === 0 || insBusy}>
              📤 Extract Selected ({insSelected.length})
            </button>
          </div>
        {:else if !insBusy}
          <div class="ins-empty">
            <p>Click <strong>🔬 Full Scan</strong> to analyze this archive</p>
          </div>
        {/if}

        <!-- Preview -->
        {#if insPreviewName}
          <div class="preview-panel">
            <div class="preview-header">
              <strong>{insPreviewName}</strong>
              <button class="btn-small" onclick={() => insPreviewName = ""}>✕</button>
            </div>
            <pre class="preview-content">{insPreviewContent}</pre>
          </div>
        {/if}

        <!-- Hash Results -->
        {#if insHashes.length > 0}
          <div class="hash-results">
            <h4>🔐 Batch Hash Results ({insHashes.length} files)</h4>
            <pre class="hash-table">{#each insHashes as h}{h.filename}
  MD5:   {h.md5}
  SHA1:  {h.sha1}
  SHA256:{h.sha256}

{/each}</pre>
          </div>
        {/if}
      {/if}

      <!-- Forensic Password Modal -->
      {#if forensicPwOpen}
        <div class="modal-overlay" onclick={() => forensicPwOpen=false} onkeydown={(e) => e.key === 'Escape' && (forensicPwOpen=false)} role="dialog" tabindex="-1">
          <div class="modal" onclick={(e) => e.stopPropagation()} role="document">
            <h3>🔐 Password Required</h3>
            <p class="hint">This archive is password protected</p>
            <input type="password" bind:value={forensicPw} placeholder="Enter password" class="input" />
            <div class="modal-btns">
              <button class="btn-secondary" onclick={() => forensicPwOpen=false}>Cancel</button>
              <button class="btn-primary" onclick={submitForensicPw}>Open</button>
            </div>
          </div>
        </div>
      {/if}
    </section>

  <!-- ═══ ABOUT ═══ -->
  {:else if tab === 3}
    <section class="page about">
      <div class="about-icon">
        <!-- ZipLoom App Icon — matches src-tauri/icons/icon.png -->
        <svg width="96" height="96" viewBox="0 0 96 96" xmlns="http://www.w3.org/2000/svg">
          <!-- Outer squircle background -->
          <rect x="4" y="4" width="88" height="88" rx="20" ry="20" fill="#0071e3"/>
          <!-- Inner clipboard sheet -->
          <rect x="14" y="24" width="68" height="60" rx="8" ry="8" fill="#e8eef7"/>
          <!-- Top tab (clip) -->
          <rect x="22" y="18" width="36" height="10" rx="5" ry="5" fill="#f0f4fa"/>
          <!-- Zipper pattern — vertical spine with chevrons -->
          <line x1="48" y1="30" x2="48" y2="76" stroke="#0071e3" stroke-width="1.8" opacity="0.35"/>
          <!-- Left chevron row -->
          <line x1="48" y1="32" x2="38" y2="38" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <line x1="48" y1="40" x2="38" y2="46" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <line x1="48" y1="48" x2="38" y2="54" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <line x1="48" y1="56" x2="38" y2="62" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <line x1="48" y1="64" x2="38" y2="70" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <!-- Right chevron row -->
          <line x1="48" y1="36" x2="58" y2="42" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <line x1="48" y1="44" x2="58" y2="50" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <line x1="48" y1="52" x2="58" y2="58" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <line x1="48" y1="60" x2="58" y2="66" stroke="#0071e3" stroke-width="2.2" stroke-linecap="round"/>
          <!-- ZL text -->
          <text x="48" y="72" text-anchor="middle" font-family="system-ui, -apple-system, sans-serif" font-weight="800" font-size="13" fill="#0071e3" opacity="0.65">ZL</text>
        </svg>
      </div>
      <h2>ZipLoom</h2>
      <p class="ver">Version 1.0</p>
      <p class="tagline">Archive Utility &amp; Forensic Inspector</p>
      <p class="subtitle">Pure Rust · Offline · Private</p>
      <div class="features">
        <p>✅ <strong>8 formats:</strong> ZIP · TAR · GZ · BZ2 · XZ · Zstandard — compress &amp; extract</p>
        <p>✅ 7-Zip &amp; RAR — extract support</p>
        <p>✅ AES-256 encrypted archives (ZIP)</p>
        <p>✅ Zstandard (.zst/.tzst) — modern fast compression</p>
        <p>✅ Forensic analysis — magic byte verification</p>
        <p>✅ Entropy scanning &amp; anomaly detection</p>
        <p>✅ Batch hashing (MD5, SHA-1, SHA-256)</p>
        <p>✅ Archive conversion &amp; split volumes</p>
        <p>✅ 100% offline · zero data collection</p>
      </div>
      <div class="legal-banner">
        <p class="legal-title">⚠️ LEGAL DISCLAIMER</p>
        <p class="legal-text">ZipLoom is provided <strong>"AS-IS"</strong> with <strong>NO WARRANTY</strong> of any kind, express or implied.</p>
        <p class="legal-text"><strong>NOT certified</strong> for court evidence, ISO auditing, NIST validation, or any forensic standard compliance.</p>
        <p class="legal-text">All forensic analysis results are <strong>informational only</strong>. Users must independently verify all findings before use in any legal, compliance, or security context.</p>
        <p class="legal-text">This tool is intended for <strong>authorized security research and personal archive management only</strong>.</p>
      </div>
      <div class="credit">
        <p class="dev">Made with ❤️ by <strong>Yusuf Shalahuddin Al Ayyubi As Sobari</strong></p>
        <p class="studio">YSF Studio</p>
        <a href="https://ysfloom.com" target="_blank" rel="noopener" class="domain-link">ysfloom.com</a>
      </div>
      <p class="footer">© 2026 YSF Studio — All rights reserved</p>
    </section>
  {/if}

  <!-- Toast -->
  {#if msg}
    <div class="toast" class:err={msg.startsWith("❌")} role="alert">
      <span>{msg}</span>
      <button class="toast-close" onclick={() => msg=""}>✕</button>
    </div>
  {/if}

  <!-- Progress -->
  {#if busy}
    <div class="progress-bar"><div class="fill" style="width:{progress*100}%"></div></div>
    <p class="status">{status}</p>
  {/if}
</main>

<!-- Extract Password Modal -->
{#if showPW}
  <div class="modal-overlay" onclick={() => showPW=false} onkeydown={(e) => e.key === 'Escape' && (showPW=false)} role="dialog" tabindex="-1">
    <div class="modal" onclick={(e) => e.stopPropagation()} role="document">
      <h3>🔐 Password Required</h3>
      {#if errMsg}<p class="err">{errMsg}</p>{/if}
      <input type="password" bind:value={pendingPw} placeholder="Enter password" class="input" />
      <div class="modal-btns">
        <button class="btn-secondary" onclick={() => showPW=false}>Cancel</button>
        <button class="btn-primary" onclick={extractWithPW}>Extract</button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(*) { margin: 0; padding: 0; box-sizing: border-box; }
  :global(body) { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: var(--bg); color: var(--fg); user-select: none; -webkit-user-select: none; }
  :root { --bg: #f5f5f7; --fg: #1d1d1f; --card: #fff; --border: #d2d2d7; --accent: #0071e3; --accent-hover: #0077ed; --danger: #ff3b30; --warn: #ff9f0a; --success: #30d158; }
  @media (prefers-color-scheme: dark) { :root { --bg: #1c1c1e; --fg: #f5f5f7; --card: #2c2c2e; --border: #3a3a3c; } }
  .app { max-width: 1000px; margin: 0 auto; padding: 20px; min-height: 100vh; }
  .header { display: flex; align-items: center; gap: 20px; margin-bottom: 24px; padding-bottom: 16px; border-bottom: 1px solid var(--border); }
  .header h1 { font-size: 22px; font-weight: 700; }
  .tabs { display: flex; gap: 4px; flex-wrap: wrap; }
  .tab { padding: 6px 16px; border: 1px solid var(--border); background: var(--card); border-radius: 8px; cursor: pointer; font-size: 13px; color: var(--fg); }
  .tab.active { background: var(--accent); color: white; border-color: var(--accent); }
  .page { display: flex; flex-direction: column; gap: 16px; }
  .dropzone { border: 2px dashed var(--border); border-radius: 16px; padding: 40px; text-align: center; cursor: pointer; background: var(--card); }
  .dropzone:hover { border-color: var(--accent); }
  .drop-icon { font-size: 48px; margin-bottom: 8px; }
  .file-list { display: flex; flex-direction: column; gap: 4px; }
  .file-row { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: var(--card); border-radius: 8px; border: 1px solid var(--border); font-size: 13px; }
  .btn-small { background: none; border: none; cursor: pointer; color: var(--danger); font-size: 16px; padding: 2px 6px; }
  .options { display: flex; flex-direction: column; gap: 12px; padding: 16px; background: var(--card); border-radius: 12px; border: 1px solid var(--border); font-size: 14px; }
  .options label { display: flex; align-items: center; gap: 8px; }
  .input { padding: 8px 12px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--fg); font-size: 13px; width: 100%; }
  select.input { cursor: pointer; }
  .btn-primary { padding: 10px 24px; background: var(--accent); color: white; border: none; border-radius: 10px; font-size: 14px; font-weight: 600; cursor: pointer; }
  .btn-primary:hover { background: var(--accent-hover); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-secondary { padding: 8px 16px; background: var(--card); color: var(--fg); border: 1px solid var(--border); border-radius: 8px; font-size: 13px; cursor: pointer; }
  .btn-row { display: flex; gap: 8px; }
  .opt-group { border-top: 1px solid var(--border); padding-top: 12px; margin-top: 4px; }
  .opt-group h4 { font-size: 13px; margin-bottom: 8px; color: #86868b; }
  .inline-row { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
  .badge { display: inline-block; padding: 2px 8px; background: color-mix(in srgb, var(--accent) 10%, var(--card)); border-radius: 4px; font-size: 11px; color: var(--accent); }
  .err { color: var(--danger); font-size: 13px; }
  .hint { font-size: 11px; color: #86868b; margin-top: 4px; }
  .level-row { display: flex; align-items: center; gap: 10px; }
  .level-slider { flex: 1; height: 4px; }
  .level-label { font-size: 13px; min-width: 120px; }
  .level-hint { font-size: 11px; color: #86868b; min-width: 50px; text-align: right; }
  .result-card { background: color-mix(in srgb, var(--success) 10%, var(--card)); border: 1px solid var(--success); border-radius: 12px; padding: 16px; }
  .result-text { font-family: monospace; font-size: 11px; white-space: pre-wrap; line-height: 1.6; }
  .about { text-align: center; padding-top: 40px; gap: 12px; }
  .about-icon { margin: 0 auto; } .about-icon svg { display: block; margin: 0 auto; }
  .ver { color: #86868b; font-size: 13px; }
  .tagline { font-size: 16px; font-weight: 500; }
  .subtitle { font-size: 12px; color: #86868b; }
  .features { text-align: left; margin: 16px auto; font-size: 14px; line-height: 2; max-width: 400px; }
  .credit { margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border); }
  .dev { font-size: 13px; }
  .studio { font-size: 12px; color: #86868b; margin-top: 4px; }
  .domain-link { display: inline-block; margin-top: 8px; padding: 4px 16px; border: 1px solid var(--accent); border-radius: 20px; color: var(--accent); text-decoration: none; font-size: 12px; font-weight: 500; }
  .domain-link:hover { background: var(--accent); color: white; }
  .footer { color: #86868b; font-size: 11px; margin-top: 16px; }
  .legal-banner { margin-top: 16px; padding: 12px; background: color-mix(in srgb, var(--warn) 10%, var(--card)); border: 1px solid var(--warn); border-radius: 10px; text-align: left; }
  .legal-title { font-size: 12px; font-weight: 700; color: var(--warn); margin-bottom: 4px; }
  .legal-text { font-size: 11px; color: #86868b; line-height: 1.5; }
  .toast { position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%); padding: 12px 24px; background: var(--card); border: 1px solid var(--border); border-radius: 12px; font-size: 13px; z-index: 100; display: flex; align-items: center; gap: 12px; }
  .toast.err { border-color: var(--danger); }
  .toast-close { background: none; border: none; font-size: 16px; cursor: pointer; color: var(--fg); opacity: 0.5; }
  .clear-btn { background: none; border: none; font-size: 14px; cursor: pointer; color: var(--danger); opacity: 0.6; margin-left: 8px; padding: 0 4px; vertical-align: middle; }
  .progress-bar { height: 4px; background: var(--border); border-radius: 2px; overflow: hidden; }
  .fill { height: 100%; background: var(--accent); transition: width 0.3s; }
  .status { text-align: center; font-size: 12px; color: #86868b; }
  .modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 200; }
  .modal { background: var(--card); border-radius: 16px; padding: 24px; width: 320px; display: flex; flex-direction: column; gap: 16px; }
  .modal h3 { text-align: center; }
  .modal-btns { display: flex; gap: 8px; justify-content: flex-end; }
  /* Inspect */
  .ins-toolbar { display: flex; gap: 8px; flex-wrap: wrap; }
  .ins-controls { display: flex; align-items: center; gap: 4px; font-size: 12px; }
  .btn-small-ins { padding: 4px 12px; border: 1px solid var(--border); background: var(--card); border-radius: 6px; cursor: pointer; font-size: 12px; color: var(--fg); }
  .btn-small-ins.active { background: var(--accent); color: white; border-color: var(--accent); }
  .file-count { margin-left: auto; color: #86868b; font-size: 12px; }
  .col-toggles { display: flex; gap: 12px; flex-wrap: wrap; font-size: 11px; padding: 4px 0; }
  .col-toggles label { display: flex; align-items: center; gap: 4px; cursor: pointer; color: #86868b; }
  .anomalies { background: color-mix(in srgb, var(--warn) 10%, var(--card)); border: 1px solid var(--warn); border-radius: 10px; padding: 12px; }
  .anomalies h4 { font-size: 13px; margin-bottom: 8px; }
  .anomaly-item { font-size: 12px; padding: 2px 0; }
  .ins-table-wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: 10px; }
  .ins-table { font-size: 12px; }
  .ins-header, .ins-row, .tree-row { display: flex; align-items: center; padding: 5px 8px; gap: 8px; }
  .ins-header { background: var(--border); font-weight: 600; font-size: 11px; color: #86868b; }
  .ins-body { max-height: 400px; overflow-y: auto; }
  .ins-row { border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent); }
  .ins-row:hover { background: color-mix(in srgb, var(--accent) 5%, var(--card)); }
  .ins-row.selected { background: color-mix(in srgb, var(--accent) 10%, var(--card)); }
  .tree-row { border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent); font-weight: 500; color: var(--accent); }
  .col-name { flex: 1; min-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; cursor: pointer; }
  .col-name:hover { color: var(--accent); }
  .col-size { width: 70px; text-align: right; flex-shrink: 0; }
  .col-hash { width: 65px; text-align: left; font-family: monospace; font-size: 10px; flex-shrink: 0; }
  .col-entropy { width: 65px; text-align: right; font-family: monospace; font-size: 11px; flex-shrink: 0; }
  .col-magic { width: 40px; text-align: center; flex-shrink: 0; }
  .col-time { width: 85px; text-align: left; font-size: 10px; flex-shrink: 0; }
  .hash-short { color: #86868b; }
  .flag-high { color: var(--danger); font-size: 12px; margin-left: 2px; }
  .flag-mid { color: var(--warn); font-size: 12px; margin-left: 2px; }
  .ins-actions { display: flex; gap: 8px; }
  .ins-empty { text-align: center; padding: 40px; color: #86868b; font-size: 14px; }
  .preview-panel { border: 1px solid var(--border); border-radius: 10px; overflow: hidden; }
  .preview-header { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: var(--border); font-size: 12px; }
  .preview-content { padding: 12px; font-family: monospace; font-size: 11px; max-height: 250px; overflow: auto; white-space: pre-wrap; background: var(--bg); }
  .hash-results { border: 1px solid var(--border); border-radius: 10px; overflow: hidden; }
  .hash-results h4 { padding: 8px 12px; background: var(--border); font-size: 12px; }
  .hash-table { padding: 12px; font-family: monospace; font-size: 10px; max-height: 300px; overflow: auto; white-space: pre; background: var(--bg); }
</style>
