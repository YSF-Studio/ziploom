/**
 * GUI smoke test — Playwright + mocked Tauri IPC
 * Run: node tests/gui-smoke.mjs  (starts vite preview automatically)
 */
import { chromium } from "playwright";
import { spawn } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const FIXTURE = path.join(ROOT, "tests/fixtures/e2e");
const OUT = path.join("/tmp", `ziploom-gui-${process.pid}`);

const results = [];
const pass = (name) => results.push({ name, ok: true });
const fail = (name, err) => results.push({ name, ok: false, err: String(err) });

function mockInitScript(fixture, outDir) {
  return `
    globalThis.isTauri = true;
    window.__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: 'main' },
        currentWebview: { windowLabel: 'main', label: 'main' }
      },
      transformCallback: (cb) => {
        const id = '_' + Math.random().toString(36).slice(2);
        window[id] = cb;
        return id;
      },
      convertFileSrc: (p) => 'asset://localhost/' + encodeURIComponent(p),
      invoke: async (cmd, args) => {
        const fixture = ${JSON.stringify(fixture)};
        const outDir = ${JSON.stringify(outDir)};

        if (cmd === 'plugin:event|listen') return args?.handler ?? 1;
        if (cmd === 'plugin:event|unlisten') return true;

        if (cmd === 'plugin:dialog|open') {
          const o = args?.options || {};
          if (o.directory && o.multiple === false) {
            if (o.title && o.title.includes('ekstraksi')) return outDir + '/extracted';
            return outDir;
          }
          if (o.title && o.title.includes('Simpan')) return outDir + '/test_bundle.zip';
          if (o.directory === true) return fixture + '/nested';
          if (o.multiple) {
            return [
              fixture + '/sample_alpha.txt',
              fixture + '/sample_beta.txt',
              fixture + '/nested'
            ];
          }
          return fixture + '/sample_alpha.txt';
        }
        if (cmd === 'plugin:dialog|save') {
          const dp = args?.options?.defaultPath || '';
          if (dp.includes('archive') || args?.options?.title?.includes('Simpan')) {
            return outDir + '/' + (dp || 'archive.zip');
          }
          return outDir + '/ziploom-inspect.csv';
        }
        if (cmd === 'plugin:fs|write_text_file') return true;
        if (cmd === 'archive_needs_password') return false;

        if (cmd === 'stat_paths') {
          return (args.paths || []).map((p) => ({
            path: p,
            isDir: p.endsWith('nested'),
            size: p.endsWith('nested') ? 4096 : 128
          }));
        }
        if (cmd === 'compress_files') {
          const pw = args.password ? ' (password-protected)' : '';
          return {
            success: true,
            outputPath: args.output,
            filesProcessed: (args.sources || []).length,
            totalSize: 1024,
            message: 'Compressed OK' + pw
          };
        }
        if (cmd === 'inspect_archive') {
          return {
            format: 'zip',
            total_files: 3,
            total_size: 384,
            total_compressed: 200,
            entries: [
              { path: 'sample_alpha.txt', size: 128, compressed_size: 80, is_dir: false, modified: '2026-01-01' },
              { path: 'sample_beta.txt', size: 128, compressed_size: 80, is_dir: false, modified: '2026-01-02' },
              { path: 'nested/sample_gamma.txt', size: 128, compressed_size: 40, is_dir: false, modified: '2026-01-03' }
            ]
          };
        }
        if (cmd === 'extract_archive') {
          return {
            success: true,
            output_path: args.outputDir,
            files_processed: 3,
            total_size: 384,
            message: 'Extracted OK'
          };
        }
        if (cmd === 'forensic_scan_archive') {
          return {
            format: 'zip',
            total_files: 3,
            total_size: 384,
            risk_score: 0,
            risk_label: 'Clean',
            threats: [],
            anomalies: [],
            entries: [
              {
                path: 'sample_alpha.txt', size: 128, is_dir: false,
                entropy: 4.2, magic_match: true, detected_type: null,
                md5: 'a'.repeat(32), sha1: '1'.repeat(40), sha256: 'b'.repeat(64)
              },
              {
                path: 'sample_beta.txt', size: 128, is_dir: false,
                entropy: 4.1, magic_match: true, detected_type: null,
                md5: 'c'.repeat(32), sha1: '2'.repeat(40), sha256: 'd'.repeat(64)
              },
              {
                path: 'nested/sample_gamma.txt', size: 128, is_dir: false,
                entropy: 4.0, magic_match: true, detected_type: null,
                md5: 'e'.repeat(32), sha1: '3'.repeat(40), sha256: 'f'.repeat(64)
              }
            ]
          };
        }
        if (cmd === 'archive_needs_password') return false;
        if (cmd === 'preview_archive_entry') {
          return {
            path: args.entryPath,
            size: 128,
            truncated: false,
            preview_type: 'text',
            text: 'sample preview content',
            hex: null,
            image_base64: null,
            mime_type: 'text/plain',
            warning: null,
            safe: true
          };
        }
        if (cmd === 'get_progress') {
          return {
            percent: 100,
            status: 'Complete',
            is_done: true,
            error: null,
            eta_secs: null,
            bytes_processed: 384,
            total_bytes: 384
          };
        }
        if (cmd === 'hash_archive') {
          return {
            md5: 'aa'.repeat(16),
            sha1: 'bb'.repeat(20),
            sha256: 'cc'.repeat(32)
          };
        }
        if (cmd === 'extract_archive_entries') {
          return {
            success: true,
            output_path: args.outputDir,
            files_processed: (args.paths || []).length,
            total_size: 256,
            message: 'Extracted selected OK'
          };
        }
        if (cmd === 'test_archive_integrity') return true;
        if (cmd === 'hash_file_sha256') return 'deadbeef'.repeat(8);
        if (cmd === 'check_tools') {
          return [
            { name: 'zip', available: true },
            { name: 'tar', available: true }
          ];
        }
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
    const proc = spawn("npx", ["vite", "preview", "--port", "1422", "--strictPort"], {
      cwd: ROOT,
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env, FORCE_COLOR: "0" },
    });
    proc.on("error", reject);
    waitForServer("http://localhost:1422/")
      .then(() => resolve(proc))
      .catch(reject);
  });
}

async function run() {
  fs.mkdirSync(OUT, { recursive: true });

  let preview;
  let browser;
  try {
    preview = await startPreview();
    await new Promise((r) => setTimeout(r, 1500));

    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage();
    await page.addInitScript(mockInitScript(FIXTURE, OUT));
    await page.goto("http://localhost:1422/", { waitUntil: "networkidle" });

    // ── Shell ──
    await page.waitForSelector(".brand .title", { text: "ZipLoom" });
    pass("App shell loads");

    const tabstrip = page.locator(".tabstrip");
    for (const tab of ["Compress", "Extract", "Inspect", "About"]) {
      await tabstrip.getByRole("button", { name: tab, exact: true }).click();
      await page.waitForTimeout(200);
      pass(`Tab ${tab} switches`);
    }

    // ── Theme toggle ──
    await page.locator(".theme-toggle-btn").click();
    pass("Theme toggle cycles");

    // ── Compress ──
    await tabstrip.getByRole("button", { name: "Compress", exact: true }).click();
    await page.locator(".compress-page .dropzone-lg").waitFor();
    await page.locator(".compress-page .dropzone-lg .dz-cta", { hasText: "Browse files" }).click();
    await page.waitForTimeout(400);
    await page.locator(".source-chip", { hasText: "sample_alpha.txt" }).waitFor({ timeout: 5000 });
    pass("Compress: browse adds files");

    await page.locator(".compress-page .dropzone-lg").click();
    await page.waitForTimeout(200);
    pass("Compress: dropzone is clickable");

    await page.locator(".compress-page .btn-cta").click();
    await page.waitForSelector(".toast.success", { timeout: 5000 });
    pass("Compress: compress action shows success toast");

    // ── Extract ──
    await tabstrip.getByRole("button", { name: "Extract", exact: true }).click();
    await page.locator(".extract-page .dropzone-lg").click();
    await page.waitForTimeout(300);
    await page.locator(".extract-page .btn-cta").click();
    await page.waitForSelector(".toast.success", { timeout: 5000 });
    pass("Extract: extract flow works");

    // ── Inspect ──
    await tabstrip.getByRole("button", { name: "Inspect", exact: true }).click();
    await page.locator(".inspect-page .dropzone-lg").click();
    await page.waitForTimeout(600);
    await page.locator(".inspect-page .archive-bar").waitFor({ timeout: 5000 });
    await page.locator(".inspect-page .inspect-table td.name", { hasText: "sample_alpha.txt" }).first().waitFor({ timeout: 5000 });
    pass("Inspect: archive loads table");

    await page.locator(".inspect-page .action-chip", { hasText: "Full Scan" }).click();
    await page.waitForSelector(".toast.success", { timeout: 5000 });
    pass("Inspect: full scan works");

    await page.locator(".inspect-page .action-chip", { hasText: "Hash Archive" }).click();
    await page.waitForSelector(".toast.success", { timeout: 5000 });
    pass("Inspect: hash all works");

    await page.locator(".inspect-page .action-chip", { hasText: "CSV" }).click();
    await page.waitForSelector(".toast.success", { timeout: 5000 });
    pass("Inspect: export CSV works");

    // ── About ──
    await tabstrip.getByRole("button", { name: "About", exact: true }).click();
    await page.waitForSelector(".about h1", { text: "ZipLoom" });
    await page.waitForSelector(".about .disclaimer");
    pass("About page renders");

  } catch (e) {
    fail("GUI smoke", e);
  } finally {
    if (browser) await browser.close();
    if (preview) preview.kill("SIGTERM");
    try { fs.rmSync(OUT, { recursive: true, force: true }); } catch {}
  }

  const failed = results.filter((r) => !r.ok);
  console.log("\n=== ZipLoom GUI Smoke Test ===\n");
  for (const r of results) {
    console.log(r.ok ? `  PASS  ${r.name}` : `  FAIL  ${r.name}: ${r.err}`);
  }
  console.log(`\n${results.length - failed.length}/${results.length} passed\n`);
  process.exit(failed.length ? 1 : 0);
}

run();
