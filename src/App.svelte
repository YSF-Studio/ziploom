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

  function selectTab(i) {
    activeTab = i;
    toast = { message: "", type: "info" };
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
      setTab: (i) => { selectTab(i); },
      setSources: (paths) => compressRef?.addPaths(paths),
      setInspectArchive: (path) => inspectRef?.inspectPath(path),
      setExtractResult: (path) => extractRef?.setArchive?.(path),
      setMsg: (msg, type) => showToast(msg, type),
    };
  }
</script>

<svelte:head>
  <title>ZipLoom</title>
</svelte:head>

<div class="app-shell">
  <header class="appbar">
    <div class="brand">
      <Logo size={28} />
      <div class="brand-copy">
        <span class="title">ZipLoom</span>
        <span class="subtitle">Archive Utility</span>
      </div>
    </div>

    <nav class="tabstrip" aria-label="Main">
      {#each tabs as tab, i}
        <button class:active={activeTab === i} onclick={() => selectTab(i)}>
          {tab}
        </button>
      {/each}
    </nav>

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

<style>
  .app-shell {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background:
      radial-gradient(circle at top left, color-mix(in srgb, var(--accent) 10%, transparent), transparent 28%),
      linear-gradient(180deg, var(--bg) 0%, color-mix(in srgb, var(--bg) 94%, var(--surface)) 100%);
  }
  .appbar {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 16px;
    padding: 12px 18px;
    border-bottom: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface) 86%, transparent);
    backdrop-filter: blur(14px);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .brand-copy {
    display: flex;
    flex-direction: column;
    line-height: 1.1;
    min-width: 0;
  }
  .title {
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.03em;
    color: var(--text);
  }
  .subtitle {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .tabstrip {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 8px;
    min-width: 0;
  }
  .tabstrip button,
  .theme-toggle-btn {
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--muted);
    border-radius: 999px;
    font-size: 14px;
    cursor: pointer;
    transition: transform 120ms ease, background 120ms ease, color 120ms ease, border-color 120ms ease, box-shadow 120ms ease;
  }
  .tabstrip button {
    padding: 10px 20px;
    min-width: 112px;
    box-shadow: 0 1px 0 rgba(255,255,255,0.45) inset, 0 8px 20px rgba(0,0,0,0.04);
  }
  .tabstrip button.active {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
    box-shadow: 0 10px 24px color-mix(in srgb, var(--accent) 26%, transparent);
  }
  .theme-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    justify-self: end;
    white-space: nowrap;
  }
  .tabstrip button:hover,
  .theme-toggle-btn:hover {
    transform: translateY(-1px);
  }
  .workspace {
    flex: 1;
    min-height: 0;
    padding: 14px 18px 0;
  }
  .workspace-main {
    height: 100%;
    min-height: 0;
  }
  .statusbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 10px 18px 12px;
    border-top: 1px solid var(--border);
    background: color-mix(in srgb, var(--surface) 92%, transparent);
    color: var(--muted);
    font-size: 12px;
  }
  .offline-badge {
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--ok-soft);
    color: var(--ok);
    font-weight: 700;
    letter-spacing: 0.02em;
  }
  :global(body) {
    margin: 0;
    background: var(--bg);
  }
  :global(button:focus-visible) {
    outline: 2px solid color-mix(in srgb, var(--accent) 70%, white);
    outline-offset: 2px;
  }
  @media (max-width: 900px) {
    .appbar {
      grid-template-columns: 1fr;
      justify-items: stretch;
    }
    .brand, .theme-toggle-btn {
      justify-self: stretch;
    }
    .tabstrip {
      justify-content: stretch;
    }
    .tabstrip button {
      flex: 1 1 0;
      min-width: 0;
    }
  }
</style>
