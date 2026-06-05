<script>
  let {
    variant = "files",
    title = "Drop files here",
    hint = "",
    browseLabel = "Choose files",
    changeLabel = "Click to change",
    fileName = "",
    badge = "",
    disabled = false,
    onBrowse,
    onBrowseFolder,
    folderLabel = "Browse folder",
    onClear,
    ondrop,
  } = $props();

  let dragging = $state(false);

  const empty = $derived(!fileName);

  function handleClick() {
    if (disabled) return;
    onBrowse?.();
  }

  function onDragOver(e) {
    e.preventDefault();
    if (!disabled) dragging = true;
  }

  function onDragLeave() {
    dragging = false;
  }

  function onDropHandler(e) {
    dragging = false;
    e.preventDefault();
    if (!disabled) ondrop?.(e);
  }
</script>

<button
  type="button"
  class="dropzone-lg"
  class:empty
  class:filled={!empty}
  class:dragging
  {disabled}
  aria-label={empty ? browseLabel : `${fileName}. ${changeLabel}`}
  onclick={handleClick}
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDropHandler}
>
  {#if empty}
    <div class="dz-icon-wrap" aria-hidden="true">
      {#if variant === "search"}
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
      {:else if variant === "archive"}
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
      {:else}
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
      {/if}
    </div>

    <p class="dz-title">{title}</p>
    <div class="dz-actions">
      <span class="dz-cta">{browseLabel}</span>
      {#if onBrowseFolder}
        <span class="dz-sep">·</span>
        <span
          class="dz-link"
          role="button"
          tabindex="0"
          onclick={(e) => { e.stopPropagation(); onBrowseFolder(); }}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.stopPropagation();
              e.preventDefault();
              onBrowseFolder();
            }
          }}
        >{folderLabel}</span>
      {/if}
    </div>
    {#if hint}<p class="dz-hint">{hint}</p>{/if}
  {:else}
    <div class="dz-icon-wrap small" aria-hidden="true">
      {#if variant === "search"}
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
      {:else}
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
      {/if}
    </div>

    <div class="dz-file">
      <span class="name">{fileName}</span>
      {#if onClear}
        <span
          class="btn-remove"
          role="button"
          tabindex="0"
          aria-label="Remove"
          onclick={(e) => { e.stopPropagation(); onClear(); }}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.stopPropagation();
              e.preventDefault();
              onClear();
            }
          }}
        >&times;</span>
      {/if}
    </div>

    {#if badge}<span class="badge">{badge}</span>{/if}
    <span class="dz-change">{changeLabel}</span>
  {/if}
</button>
