<script>
import { invoke } from "@tauri-apps/api/core";
let info = $state({ features: [], appName: "CollectionLoom" });
let loaded = $state(false);

async function load() {
    if (loaded) return;
    try {
        info = await invoke("about_info");
        loaded = true;
    } catch(e) { /* use defaults */ }
}
$effect(() => { load(); });
</script>

<div class="about" style="max-width:580px;margin:0 auto">
    <div style="text-align:center;margin-bottom:24px">
        <div style="font-size:48px;margin-bottom:8px">🟢</div>
        <h3 style="margin:0 0 4px;font-size:20px">
            {info.appName}
            <span style="color:var(--text-muted);font-size:12px;margin-left:8px">v{info.version}</span>
        </h3>
        <p style="color:var(--text-secondary);font-size:13px;margin:0">Portable Forensic Acquisition Toolkit — ISO 27037 · 17025</p>
    </div>

    <div class="card" style="margin-bottom:12px">
        <h4>🚀 Features</h4>
        <ul style="margin:0;padding-left:20px">
            {#each info.features as f}
                <li style="font-size:13px;color:var(--text-secondary);margin-bottom:6px;line-height:1.4">{f}</li>
            {/each}
        </ul>
    </div>

    <div class="card" style="margin-bottom:12px;border-left:3px solid var(--success)">
        <h4>🔒 Privacy</h4>
        <p style="font-size:13px;color:var(--text-secondary);margin:0;line-height:1.5">{info.privacy}</p>
        <span class="offline-badge" style="margin-top:8px">✅ Fully Offline</span>
    </div>

    <div class="card" style="margin-bottom:12px;border-left:3px solid var(--warn)">
        <h4>⚖️ Disclaimer</h4>
        <p style="font-size:13px;color:var(--text-secondary);margin:0;font-style:italic">{info.disclaimer}</p>
    </div>

    <div class="card" style="margin-bottom:12px">
        <h4>👨‍💻 Developer</h4>
        <p style="font-size:13px;color:var(--text);margin:0 0 4px;font-weight:600">{info.developer}</p>
        <p style="font-size:12px;color:var(--primary);margin:0">{info.build}</p>
    </div>

    <p style="text-align:center;font-size:11px;color:var(--text-muted);margin-top:12px">
        YSF Studio © {new Date().getFullYear()} — All rights reserved.
    </p>
</div>
