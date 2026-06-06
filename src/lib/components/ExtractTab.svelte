<script>
  import DropzoneArea from "./DropzoneArea.svelte";
  import PasswordDialog from "./PasswordDialog.svelte";
  import { invoke, open } from "../tauri.js";
  import { formatSize } from "../format.js";
  import { basename, archiveBadge } from "../path.js";
  import { notify } from "../toast.js";
  import { promptPassword, withArchivePassword } from "../password.js";

  let { onToast, onBusy } = $props();

  let archive = $state("");
  let format = $state("");
  let busy = $state(false);
  let result = $state(null);
  let cleanMac = $state(true);

  let pwOpen = $state(false);
  let pwHandlers = $state(null);

  function askPassword() {
    return promptPassword(({ resolve, cancel }) => {
      pwHandlers = { resolve, cancel };
      pwOpen = true;
    });
  }

  export function setArchive(path) {
    if (!path) return;
    archive = path;
    format = "";
    result = null;
  }

  export async function extractPath(path) {
    setArchive(path);
    await extract();
  }

  async function browse() {
    const sel = await open({ directory: false, multiple: false });
    if (sel) {
      archive = sel;
      format = "";
      result = null;
    }
  }

  async function extract() {
    if (!archive) return notify(onToast, "Select an archive first", "error");

    const dir = await open({ directory: true, multiple: false, title: "Extraction destination folder" });
    if (!dir) return;

    busy = true;
    onBusy?.(true);
    result = null;
    try {
      const res = await withArchivePassword(
        archive,
        invoke,
        askPassword,
        (pw) => invoke("extract_archive", { archivePath: archive, outputDir: dir, password: pw })
      );
      result = res;
      notify(onToast, res.message, "success");
    } catch (e) {
      notify(onToast, String(e), "error");
    } finally {
      busy = false;
      onBusy?.(false);
    }
  }

  function clear() {
    archive = "";
    result = null;
    format = "";
  }

  function onDrop(e) {
    e.preventDefault();
    const p = e.dataTransfer?.files?.[0]?.path;
    if (p) {
      archive = p;
      result = null;
    }
  }
</script>

<div class="page extract-page">
  <DropzoneArea
    variant="archive"
    title="Drop archive here or click to browse"
    browseLabel="Choose archive"
    changeLabel="Click to change archive"
    fileName={archive ? basename(archive) : ""}
    badge={archive ? archiveBadge(archive, format) : ""}
    disabled={busy}
    onBrowse={browse}
    onClear={clear}
    ondrop={onDrop}
  />

  {#if archive}
    <div class="options-strip">
      <label>
        <input type="checkbox" bind:checked={cleanMac} disabled={busy} />
        Remove __MACOSX/ and .DS_Store
      </label>
    </div>

    <button class="btn-cta" disabled={busy} onclick={extract}>
      {busy ? "Extracting…" : "Extract"}
    </button>

    {#if result}
      <div class="result" style="margin-top:14px">
        {result.filesProcessed ?? result.files_processed} files ({formatSize(result.totalSize ?? result.total_size)}) extracted to {result.outputPath ?? result.output_path}
      </div>
    {/if}
  {/if}
</div>

<PasswordDialog
  bind:open={pwOpen}
  title="Encrypted archive"
  message="This archive is password protected. Enter the password to extract."
  onConfirm={(pw) => pwHandlers?.resolve(pw)}
  onCancel={() => pwHandlers?.cancel()}
/>
