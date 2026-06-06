<script>
  import { initTheme, setTheme } from "./lib/theme.js";
  import { getPrefs, savePrefs } from "./lib/prefs.js";
  import { setupDragDrop } from "./lib/tauri.js";
  import Logo from "./lib/Logo.svelte";
  import Toast from "./lib/components/Toast.svelte";
  import CompressTab from "./lib/components/CompressTab.svelte";
  import ExtractTab from "./lib/components/ExtractTab.svelte";
  import InspectTab from "./lib/components/InspectTab.svelte";
  import AboutTab from "./lib/components/AboutTab.svelte";
  import { getWindow } from "./lib/tauri.js";

  let activeTab = $state(0);
  let toast = $state({ message: "", type: "info" });
  let busy = $state(false);
  let prefs = $state(getPrefs());
  let compressRef = $state(null);
  let extractRef = $state(null);
  let inspectRef = $state(null);

  const tabs = ["Compress", "Extract", "Inspect", "About"];
  const themeCycle = ["light", "dark", "system"];
  const themeLabels = {
    light: "Light mode",
    dark: "Dark mode",
    system: "System default",
  };

  function cycleTheme() {
    const i = themeCycle.indexOf(prefs.theme);
    const next = themeCycle[(i + 1) % themeCycle.length];
    setTheme(next);
    prefs = savePrefs({ theme: next });
  }

  function windowAction(action) {
    const win = getWindow();
    if (!win) return;
    if (action === "close") win.close();
    else if (action === "minimize") win.minimize();
    else if (action === "maximize") win.toggleMaximize();
  }

  function showToast(message, type = "info") {
    toast = { message, type };
    if (message && type !== "error") {
      setTimeout(() => {
        if (toast.message === message) toast = { message: "", type: "info" };
      }, 6000);
    }
  }

  $effect(() => initTheme(prefs.theme));

  $effect(() => {
    return setupDragDrop(async (paths) => {
      if (!paths.length || busy) return;
      if (activeTab === 0) await compressRef?.addPaths(paths);
      else if (activeTab === 1) {
        extractRef?.setArchive?.(paths[0]);
      } else if (activeTab === 2) await inspectRef?.inspectPath(paths[0]);
    });
  });

  if (typeof window !== "undefined") {
    window.__zipLoom = {
      setTab: (i) => { activeTab = i; toast = { message: "", type: "info" }; },
      setSources: (paths) => compressRef?.addPaths(paths),
      setInspectArchive: (path) => inspectRef?.inspectPath(path),
      setExtractResult: (path) => extractRef?.setArchive?.(path),
      setMsg: (msg, type) => showToast(msg, type),
    };
  }
</script>

<div class="app-shell">
  <header class="titlebar">
    <div class="traffic-lights" aria-label="Window controls">
      <button type="button" class="tl red" aria-label="Close window" onclick={() => windowAction("close")}></button>
      <button type="button" class="tl yellow" aria-label="Minimize" onclick={() => windowAction("minimize")}></button>
      <button type="button" class="tl green" aria-label="Maximize" onclick={() => windowAction("maximize")}></button>
    </div>
    <div class="brand">
      <Logo size={28} />
      <span class="title">ZipLoom</span>
    </div>
    <nav class="tabstrip" aria-label="Main">
      {#each tabs as tab, i}
        <button class:active={activeTab === i} onclick={() => { activeTab = i; toast = { message: "", type: "info" }; }}>
          {tab}
        </button>
      {/each}
    </nav>
    <div class="titlebar-end">
      <button
        type="button"
        class="theme-toggle-btn"
        onclick={cycleTheme}
        aria-label={`Theme: ${themeLabels[prefs.theme] ?? prefs.theme}. Click to change.`}
        title={`Theme: ${themeLabels[prefs.theme] ?? prefs.theme}`}
      >
        {#if prefs.theme === "dark"}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        {:else if prefs.theme === "system"}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>
          </svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
          </svg>
        {/if}
        <span>{themeLabels[prefs.theme] ?? prefs.theme}</span>
      </button>
    </div>
  </header>

  <div class="workspace">
    <main class="workspace-main">
      {#if activeTab === 0}
        <CompressTab bind:this={compressRef} onToast={showToast} onBusy={(v) => (busy = v)} />
      {:else if activeTab === 1}
        <ExtractTab bind:this={extractRef} onToast={showToast} onBusy={(v) => (busy = v)} />
      {:else if activeTab === 2}
        <InspectTab bind:this={inspectRef} onToast={showToast} onBusy={(v) => (busy = v)} />
      {:else}
        <AboutTab />
      {/if}
    </main>
  </div>

  <footer class="statusbar">
    <span>{busy ? "Processing…" : "ZipLoom — Archive Utility"}</span>
    <span class="offline-badge">Offline</span>
  </footer>
</div>

<Toast message={toast.message} type={toast.type} onClose={() => (toast = { message: "", type: "info" })} />
