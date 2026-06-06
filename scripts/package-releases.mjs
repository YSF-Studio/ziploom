#!/usr/bin/env node
/**
 * After `tauri build`, collect installer + portable artifacts into
 * src-tauri/target/release/bundle/releases/ with consistent names.
 */
import { cp, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const tauriDir = join(root, 'src-tauri');

const conf = JSON.parse(await readFile(join(tauriDir, 'tauri.conf.json'), 'utf8'));
const version = conf.version;
const product = conf.productName;
const slug = product.replace(/\s+/g, '');

const targetRoot = process.env.CARGO_TARGET_DIR
  ? join(process.env.CARGO_TARGET_DIR, 'release')
  : join(tauriDir, 'target', 'release');
const bundleDir = join(targetRoot, 'bundle');
const releasesDir = join(bundleDir, 'releases');

function run(cmd, args, opts = {}) {
  const result = spawnSync(cmd, args, { stdio: 'inherit', ...opts });
  if (result.status !== 0) {
    throw new Error(`Command failed: ${cmd} ${args.join(' ')}`);
  }
}

async function findOne(dir, pattern) {
  if (!existsSync(dir)) return null;
  const entries = await readdir(dir, { withFileTypes: true });
  const re = typeof pattern === 'string' ? new RegExp(pattern) : pattern;
  const match = entries.find((e) => re.test(e.name));
  return match ? join(dir, match.name) : null;
}

async function copyArtifact(src, destName) {
  if (!src || !existsSync(src)) return null;
  await mkdir(releasesDir, { recursive: true });
  const dest = join(releasesDir, destName);
  await cp(src, dest, { recursive: true });
  console.log(`  ✓ ${destName}`);
  return dest;
}

async function zipDirectory(sourceDir, zipPath) {
  if (process.platform === 'win32') {
    run(
      'powershell',
      [
        '-NoProfile',
        '-Command',
        `Compress-Archive -Path '${sourceDir}\\*' -DestinationPath '${zipPath}' -Force`,
      ],
      { shell: false },
    );
  } else if (spawnSync('zip', ['-h'], { stdio: 'ignore' }).status === 0) {
    run('zip', ['-r', zipPath, basename(sourceDir)], { cwd: dirname(sourceDir) });
  } else {
    const tarPath = zipPath.replace(/\.zip$/i, '.tar.gz');
    run('tar', ['-czf', tarPath, '-C', dirname(sourceDir), basename(sourceDir)]);
    console.log(`  (zip unavailable, created ${basename(tarPath)} instead)`);
    return tarPath;
  }
  return zipPath;
}

async function packageMacos() {
  console.log('macOS releases:');
  const dmgDir = join(bundleDir, 'dmg');
  const appDir = join(bundleDir, 'macos');

  const dmg = await findOne(dmgDir, /\.dmg$/i);
  await copyArtifact(dmg, `${slug}_${version}_macos_installer.dmg`);

  const app = await findOne(appDir, /\.app$/i);
  if (!app) {
    console.warn('  ! ZipLoom.app not found — skip portable zip');
    return;
  }

  const staging = join(releasesDir, '_macos_portable');
  await rm(staging, { recursive: true, force: true });
  await mkdir(staging, { recursive: true });
  await cp(app, join(staging, basename(app)), { recursive: true });

  const zipPath = join(releasesDir, `${slug}_${version}_macos_portable.zip`);
  await zipDirectory(staging, zipPath);
  await rm(staging, { recursive: true, force: true });
  console.log(`  ✓ ${basename(zipPath)}`);
}

async function packageLinux() {
  console.log('Linux releases:');
  const debDir = join(bundleDir, 'deb');
  const appimageDir = join(bundleDir, 'appimage');

  const deb = await findOne(debDir, /\.deb$/i);
  await copyArtifact(deb, `${slug}_${version}_linux_installer_amd64.deb`);

  const appimage = await findOne(appimageDir, /\.AppImage$/i);
  if (appimage) {
    await copyArtifact(appimage, `${slug}_${version}_linux_portable_amd64.AppImage`);
  } else {
    console.warn('  ! AppImage not found — install librsvg2-dev and rebuild for portable AppImage');
  }

  if (!deb) return;

  const extractDir = join(releasesDir, '_linux_deb_extract');
  await rm(extractDir, { recursive: true, force: true });
  await mkdir(extractDir, { recursive: true });
  run('dpkg-deb', ['-x', deb, extractDir]);

  const portableRoot = join(releasesDir, '_linux_portable');
  await rm(portableRoot, { recursive: true, force: true });
  await mkdir(join(portableRoot, product), { recursive: true });

  const binSrc = join(extractDir, 'usr', 'bin', 'ziploom');
  if (existsSync(binSrc)) {
    await cp(binSrc, join(portableRoot, product, 'ziploom'));
    run('chmod', ['+x', join(portableRoot, product, 'ziploom')]);
  }

  await writeFile(
    join(portableRoot, product, 'README.txt'),
    [
      `${product} ${version} — portable Linux build`,
      '',
      'Run from this folder:',
      `  ./${product}/ziploom`,
      '',
      'Requires WebKitGTK 4.1 and GTK 3 on the host system.',
      'For a self-contained build, use the .AppImage file instead.',
      '',
    ].join('\n'),
  );

  const tarPath = join(releasesDir, `${slug}_${version}_linux_portable_amd64.tar.gz`);
  run('tar', ['-czf', tarPath, '-C', portableRoot, product]);
  console.log(`  ✓ ${basename(tarPath)}`);

  await rm(extractDir, { recursive: true, force: true });
  await rm(portableRoot, { recursive: true, force: true });
}

async function packageWindows() {
  console.log('Windows releases:');
  const nsisDir = join(bundleDir, 'nsis');

  const installer = await findOne(nsisDir, /-setup\.exe$/i) ?? await findOne(nsisDir, /\.exe$/i);
  await copyArtifact(installer, `${slug}_${version}_windows_installer_x64-setup.exe`);

  const exeName = 'ziploom.exe';
  const exePath = join(targetRoot, exeName);
  if (!existsSync(exePath)) {
    console.warn(`  ! ${exeName} not found — skip portable zip`);
    return;
  }

  const portableRoot = join(releasesDir, '_windows_portable', product);
  await rm(join(releasesDir, '_windows_portable'), { recursive: true, force: true });
  await mkdir(portableRoot, { recursive: true });

  const entries = await readdir(targetRoot, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isFile()) continue;
    if (/\.(exe|dll)$/i.test(entry.name)) {
      await cp(join(targetRoot, entry.name), join(portableRoot, entry.name));
    }
  }

  const resourcesDir = join(targetRoot, 'resources');
  if (existsSync(resourcesDir)) {
    await cp(resourcesDir, join(portableRoot, 'resources'), { recursive: true });
  }

  await writeFile(
    join(portableRoot, 'README.txt'),
    [
      `${product} ${version} — portable Windows build`,
      '',
      `Double-click ${exeName} to run. No installation required.`,
      'Requires Microsoft Edge WebView2 Runtime (pre-installed on Windows 10/11).',
      '',
      'To install system-wide, use the *-setup.exe installer instead.',
      '',
    ].join('\n'),
  );

  const zipPath = join(releasesDir, `${slug}_${version}_windows_portable_x64.zip`);
  await zipDirectory(portableRoot, zipPath);
  await rm(join(releasesDir, '_windows_portable'), { recursive: true, force: true });
  console.log(`  ✓ ${basename(zipPath)}`);
}

async function main() {
  if (!existsSync(bundleDir)) {
    console.error('Bundle directory not found. Run `npm run tauri:build` first.');
    process.exit(1);
  }

  await mkdir(releasesDir, { recursive: true });
  console.log(`Packaging ${product} ${version} → bundle/releases/\n`);

  if (process.platform === 'darwin') await packageMacos();
  else if (process.platform === 'linux') await packageLinux();
  else if (process.platform === 'win32') await packageWindows();
  else console.warn(`Unsupported platform: ${process.platform}`);

  console.log('\nDone. Release artifacts:');
  if (existsSync(releasesDir)) {
    const files = await readdir(releasesDir);
    for (const f of files.sort()) console.log(`  ${f}`);
  }
}

main().catch((err) => {
  console.error(err.message || err);
  process.exit(1);
});
