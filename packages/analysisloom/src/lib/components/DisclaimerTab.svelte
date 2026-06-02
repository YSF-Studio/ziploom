<script>
import { invoke } from "@tauri-apps/api/core";
let info = $state({ features: [] });
let loaded = $state(false);

async function load() {
    if (loaded) return;
    try {
        info = await invoke("about_info");
        loaded = true;
    } catch(e) { /* fallback */ }
}
$effect(() => { load(); });
</script>

<div class="about">
    <header class="hero">
        <div class="icon">🔬</div>
        <h1>{info.appName || "AnalysisLoom"}</h1>
        <span class="version">v{info.version || "0.1.0"}</span>
    </header>

    <p class="subtitle">Forensic Analysis Workstation — ISO 27042 · 17043 · 17025</p>

    <section class="card">
        <h3>🚀 Features</h3>
        <ul>
            {#each info.features as f}
                <li>{f}</li>
            {/each}
        </ul>
    </section>

    <section class="card offline-card">
        <h3>🔒 Privacy & Security</h3>
        <p>{info.privacy || "100% offline — zero data collection. No telemetry, no analytics, no external network calls."}</p>
        <div class="badge">✅ Fully Offline</div>
    </section>

    <section class="card disclaimer-card">
        <h3>⚖️ Disclaimer</h3>
        <p class="disclaimer">{info.disclaimer || "This software is provided 'AS-IS'. Results should be independently verified before use in legal proceedings."}</p>
    </section>

    <section class="card">
        <h3>👨‍💻 Developer</h3>
        <p class="dev">{info.developer || "YSF Studio — Built with ❤️ by Yusuf Shalahuddin"}</p>
        <p class="build">{info.build || "Master Build — All Features Unlocked"}</p>
    </section>

    <footer class="footer">
        <p>YSF Studio © {new Date().getFullYear()} — All rights reserved.</p>
    </footer>
</div>

<style>
.about { max-width: 640px; margin: 0 auto; padding: 20px; }
.hero { text-align: center; margin-bottom: 24px; }
.hero .icon { font-size: 48px; margin-bottom: 8px; }
.hero h1 { margin: 0; font-size: 28px; color: #e0e0e0; display: inline; }
.version { font-size: 14px; color: var(--text-secondary); margin-left: 8px; }
.subtitle { text-align: center; color: var(--text-secondary); font-size: 13px; margin-bottom: 28px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 10px; padding: 16px 20px; margin-bottom: 16px; }
.card h3 { margin: 0 0 10px; font-size: 15px; color: #ccc; }
.card ul { margin: 0; padding-left: 20px; }
.card li { font-size: 13px; margin-bottom: 6px; color: var(--text-secondary); line-height: 1.4; }
.card p { margin: 0; font-size: 13px; color: var(--text-secondary); line-height: 1.5; }
.offline-card { border-left: 3px solid var(--success); }
.disclaimer-card { border-left: 3px solid var(--warn); }
.disclaimer-card .disclaimer { font-style: italic; color: #ccc !important; }
.badge { display: inline-block; margin-top: 10px; padding: 4px 12px; background: rgba(34,197,94,0.15); color: var(--success); border-radius: 20px; font-size: 12px; font-weight: 600; }
.dev { font-weight: 600; color: #e0e0e0 !important; margin-bottom: 4px !important; }
.build { font-size: 12px !important; color: var(--primary) !important; }
.footer { text-align: center; padding-top: 8px; }
.footer p { font-size: 11px; color: var(--text-secondary); }
</style>
