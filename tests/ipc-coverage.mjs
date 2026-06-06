#!/usr/bin/env node
/**
 * Static check: frontend invoke() commands ⊆ Rust generate_handler! list.
 * Run: node tests/ipc-coverage.mjs
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(p) {
  return fs.readFileSync(path.join(ROOT, p), "utf8");
}

const libRs = read("src-tauri/src/lib.rs");
const handlerBlock = libRs.match(/generate_handler!\s*\[([\s\S]*?)\]/);
if (!handlerBlock) {
  console.error("Could not find generate_handler! in lib.rs");
  process.exit(1);
}

const registered = [...handlerBlock[1].matchAll(/commands::(\w+)/g)].map((m) => m[1]);
const commandFile = read("src-tauri/src/commands.rs");
const defined = [...commandFile.matchAll(/#\[tauri::command\]\s*\n(?:pub async fn|pub fn) (\w+)/g)].map(
  (m) => m[1],
);

const srcFiles = [];
function walk(dir) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory() && ent.name !== "node_modules") walk(full);
    else if (/\.(svelte|js)$/.test(ent.name)) srcFiles.push(full);
  }
}
walk(path.join(ROOT, "src"));

const invoked = new Set();
for (const file of srcFiles) {
  const text = fs.readFileSync(file, "utf8");
  for (const m of text.matchAll(/invoke\(\s*["'](\w+)["']/g)) invoked.add(m[1]);
}

let failed = false;

const registeredCommands = new Set(defined);
const handlerSet = new Set(registered);

console.log("=== IPC Coverage ===\n");
console.log(`Registered commands (${registeredCommands.size}): ${[...registeredCommands].sort().join(", ")}`);
console.log(`Frontend invoke calls (${invoked.size}): ${[...invoked].sort().join(", ")}\n`);

for (const cmd of invoked) {
  if (!registeredCommands.has(cmd)) {
    console.error(`  MISSING  frontend invokes '${cmd}' but no #[tauri::command] fn '${cmd}'`);
    failed = true;
  } else {
    console.log(`  OK       ${cmd}`);
  }
}

for (const cmd of defined) {
  if (!handlerSet.has(cmd)) {
    console.error(`  UNREG    '${cmd}' has #[tauri::command] but is missing from generate_handler!`);
    failed = true;
  }
}

if (failed) process.exit(1);
console.log("\nAll frontend IPC commands are registered.\n");
