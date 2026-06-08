/**
 * Capture README screenshots using Playwright + mocked Tauri IPC with real fixture paths.
 * Run: npm run screenshots
 */
import { chromium } from "playwright";
import { spawn } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const FIXTURE = path.join(ROOT, "tests/fixtures/e2e");
const SAMPLES = path.join(ROOT, "samples");
const SHOTS = path.join(ROOT, "screenshots");
const OUT = path.join("/tmp", `ziploom-shots-${process.pid}`);

function mockInitScript(fixture, samples, outDir) {
  return `
    globalThis.isTauri = true;
    window.__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
      transformCallback: (cb) => { const id = '_' + Math.random().toString(36).slice(2); window[id] = cb; return id; },
      convertFileSrc: (p) => 'asset://localhost/' + encodeURIComponent(p),
      invoke: async (cmd, args) => {
        const fixture = ${JSON.stringify(fixture)};
        const samples = ${JSON.stringify(samples)};
        const outDir = ${JSON.stringify(outDir)};
        if (cmd === 'plugin:event|listen') return args?.handler ?? 1;
        if (cmd === 'plugin:event|unlisten') return true;
        if (cmd === 'plugin:dialog|open') {
          const o = args?.options || {};
          if (o.directory && !o.multiple) {
            if (o.title?.includes('ekstraksi')) return outDir + '/extracted';
            return outDir;
          }
          if (o.directory === true) return fixture + '/nested';
          if (o.multiple) return [fixture + '/sample_alpha.txt', fixture + '/sample_beta.txt', samples + '/evidence_manifest.txt'];
          return fixture + '/sample_alpha.txt';
        }
        if (cmd === 'plugin:dialog|save') {
          const dp = args?.options?.defaultPath || '';
          if (dp.includes('archive') || args?.options?.title?.includes('Save')) return outDir + '/' + (dp || 'archive.zip');
          return outDir + '/ziploom-inspect.csv';
        }
        if (cmd === 'plugin:fs|write_text_file') return true;
        if (cmd === 'archive_needs_password') return false;
        if (cmd === 'stat_paths') {
          return (args.paths || []).map((p) => ({
            path: p,
            isDir: p.endsWith('nested') || p.endsWith('samples'),
            size: p.endsWith('nested') ? 4096 : 2048,
          }));
        }
        if (cmd === 'compress_files') {
          const pw = args.password ? ' (password-protected ZIP)' : '';
          return { success: true, outputPath: args.output, filesProcessed: 3, totalSize: 4096, message: 'Compressed 3 files' + pw };
        }
        if (cmd === 'inspect_archive') {
          return {
            format: 'zip', totalFiles: 4, totalSize: 8192, totalCompressed: 2048,
            entries: [
              { path: 'sample_alpha.txt', size: 132, compressed_size: 80, is_dir: false, modified: '2026-01-01' },
              { path: 'sample_beta.txt', size: 90, compressed_size: 70, is_dir: false, modified: '2026-01-02' },
              { path: 'nested/sample_gamma.txt', size: 64, compressed_size: 40, is_dir: false, modified: '2026-01-03' },
              { path: 'confidential_report.txt', size: 512, compressed_size: 200, is_dir: false, modified: '2026-01-04' },
            ],
          };
        }
        if (cmd === 'forensic_scan_archive') {
          return {
            format: 'zip', total_files: 4, total_size: 8192, risk_score: 0.05, risk_label: 'Low Risk',
            threats: [{ file: 'nested/sample_gamma.txt', threat: 'Hidden path depth', category: 'obfuscation', severity: 'low', detail: 'Deep nesting' }],
            anomalies: [],
            entries: [
              { path: 'sample_alpha.txt', size: 132, is_dir: false, entropy: 4.2, magic_match: true, md5: 'a'.repeat(32), sha1: '1'.repeat(40), sha256: 'b'.repeat(64) },
              { path: 'sample_beta.txt', size: 90, is_dir: false, entropy: 4.0, magic_match: true, md5: 'c'.repeat(32), sha1: '2'.repeat(40), sha256: 'd'.repeat(64) },
              { path: 'nested/sample_gamma.txt', size: 64, is_dir: false, entropy: 3.8, magic_match: true, md5: 'e'.repeat(32), sha1: '3'.repeat(40), sha256: 'f'.repeat(64) },
            ],
          };
        }
        if (cmd === 'get_progress') return { percent: 100, status: 'Complete', is_done: true, bytes_processed: 8192, total_bytes: 8192 };
        if (cmd === 'hash_archive') return { md5: 'aa'.repeat(16), sha1: 'bb'.repeat(20), sha256: 'cc'.repeat(32) };
        if (cmd === 'extract_archive') return { success: true, outputPath: args.outputDir, filesProcessed: 4, totalSize: 8192, message: 'Extracted OK' };
        if (cmd === 'extract_archive_entries') return { success: true, outputPath: args.outputDir, filesProcessed: 1, totalSize: 132, message: 'Extracted selected OK' };
        if (cmd === 'preview_archive_entry') return { path: args.entryPath, size: 132, truncated: false, preview_type: 'text', text: 'Sample forensic preview content.', safe: true, mime_type: 'text/plain' };
        if (cmd === 'about_info') return { appName: 'ZipLoom', version: '0.1.0', offline: true, features: ['Compress', 'Extract', 'Inspect'] };
        throw new Error('Unhandled invoke: ' + cmd);
      }
    };
  `;
}

async function waitForServer(url, ms = 30000) {
  const start = Date.now();
  while (Date.now() - start < ms) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {}
    await new Promise((r) => setTimeout(r, 300));
  }
  throw new Error("vite preview timeout");
}

function startPreview() {
  return new Promise((resolve, reject) => {
    const proc = spawn("npx", ["vite", "preview", "--configLoader", "runner", "--host", "127.0.0.1", "--port", "1422", "--strictPort"], {
      cwd: ROOT,
      stdio: "pipe",
    });
    proc.stderr.on("data", (d) => process.stderr.write(d));
    proc.stdout.on("data", (d) => {
      if (String(d).includes("1422")) resolve(proc);
    });
    proc.on("error", reject);
    proc.on("exit", (code) => reject(new Error("vite exited " + code)));
  });
}

async function main() {
  fs.mkdirSync(SHOTS, { recursive: true });
  fs.mkdirSync(OUT, { recursive: true });

  let preview;
  const browser = await chromium.launch();
  try {
    preview = await startPreview();
    await waitForServer("http://127.0.0.1:1422/");

    const context = await browser.newContext({
      viewport: { width: 960, height: 640 },
      deviceScaleFactor: 2,
    });
    const page = await context.newPage();
    await page.addInitScript(mockInitScript(FIXTURE, SAMPLES, OUT));
    await page.goto("http://127.0.0.1:1422/", { waitUntil: "networkidle" });

    const tabstrip = page.locator(".tabstrip");

    // ── Compress (with sources + password banner) ──
    await tabstrip.getByRole("button", { name: "Compress", exact: true }).click();
    await page.locator(".compress-page .dropzone-lg").click();
    await page.waitForTimeout(400);
    await page.locator("#pw").check();
    await page.locator('input[type="password"]').fill("demo-password");
    await page.screenshot({ path: path.join(SHOTS, "compress.png"), fullPage: false });
    console.log("  ✓ screenshots/compress.png");

    // ── Extract ──
    await tabstrip.getByRole("button", { name: "Extract", exact: true }).click();
    await page.locator(".extract-page .dropzone-lg").click();
    await page.waitForTimeout(400);
    await page.screenshot({ path: path.join(SHOTS, "extract.png"), fullPage: false });
    console.log("  ✓ screenshots/extract.png");

    // ── Inspect (loaded + scanned state) ──
    await tabstrip.getByRole("button", { name: "Inspect", exact: true }).click();
    await page.locator(".inspect-page .dropzone-lg").click();
    await page.waitForTimeout(600);
    await page.locator(".inspect-page .action-chip", { hasText: "Full Scan" }).click();
    await page.waitForTimeout(800);
    await page.screenshot({ path: path.join(SHOTS, "inspect.png"), fullPage: false });
    console.log("  ✓ screenshots/inspect.png");

    // ── Password ZIP panel (compress settings close-up) ──
    await tabstrip.getByRole("button", { name: "Compress", exact: true }).click();
    await page.locator(".settings-panel").screenshot({ path: path.join(SHOTS, "encrypt.png") });
    console.log("  ✓ screenshots/encrypt.png");

    // ── About ──
    await tabstrip.getByRole("button", { name: "About", exact: true }).click();
    await page.waitForSelector(".about h1");
    await page.waitForTimeout(300);
    await page.screenshot({ path: path.join(SHOTS, "about.png"), fullPage: false });
    console.log("  ✓ screenshots/about.png");

    await context.close();
  } finally {
    await browser.close();
    preview?.kill("SIGTERM");
  }
  console.log("\nScreenshots saved to screenshots/");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
