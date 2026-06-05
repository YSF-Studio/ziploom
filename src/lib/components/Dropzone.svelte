<script>
  let {
    title = "Drop files here or click to browse",
    hint = "",
    icon = "inbox",
    onclick,
    ondrop,
    ondragover,
  } = $props();
</script>

<div
  class="dropzone"
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={(e) => (e.key === "Enter" || e.key === " ") && onclick?.(e)}
  {ondragover}
  {ondrop}
>
  {#if icon === "search"}
    <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
  {:else if icon === "folder"}
    <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
  {:else}
    <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
  {/if}
  <strong>{title}</strong>
  {#if hint}<span>{hint}</span>{/if}
</div>
