<script>
  let {
    open = $bindable(false),
    title = "Password required",
    message = "This archive is encrypted. Enter the password to continue.",
    onConfirm,
    onCancel,
  } = $props();

  let password = $state("");

  $effect(() => {
    if (open) password = "";
  });

  function submit() {
    if (!password) return;
    onConfirm?.(password);
    open = false;
  }

  function cancel() {
    onCancel?.();
    open = false;
  }

  function onKeydown(e) {
    if (e.key === "Escape") cancel();
    if (e.key === "Enter") submit();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="pw-overlay" role="presentation" onclick={cancel}>
    <div
      class="pw-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="pw-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={onKeydown}
    >
      <h3 id="pw-title">{title}</h3>
      <p>{message}</p>
      <input
        type="password"
        bind:value={password}
        placeholder="Enter password"
        autocomplete="current-password"
      />
      <div class="pw-actions">
        <button class="btn-secondary" type="button" onclick={cancel}>Cancel</button>
        <button class="btn-cta" type="button" disabled={!password} onclick={submit}>OK</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .pw-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    padding: 20px;
    -webkit-app-region: no-drag;
  }
  .pw-modal {
    background: var(--surface, #ffffff);
    color: var(--text, #1a1a1a);
    border: 1px solid var(--border, #d8dee8);
    border-radius: var(--radius-lg, 12px);
    padding: 20px 22px;
    width: min(380px, 92vw);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
  }
  .pw-modal h3 {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 700;
    color: var(--text, #1a1a1a);
  }
  .pw-modal p {
    margin: 0 0 14px;
    font-size: 13px;
    color: var(--muted, #6b7280);
    line-height: 1.5;
  }
  .pw-modal input {
    width: 100%;
    box-sizing: border-box;
    margin-bottom: 14px;
    padding: 10px 12px;
    border-radius: var(--radius, 8px);
    border: 1px solid var(--border, #d8dee8);
    background: var(--surface-soft, #f8f9fb);
    color: var(--text, #1a1a1a);
    font-size: 14px;
  }
  .pw-modal input:focus {
    outline: none;
    border-color: var(--accent, #2b7fff);
    box-shadow: 0 0 0 3px var(--accent-soft, rgba(43, 127, 255, 0.15));
  }
  .pw-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .pw-actions .btn-cta {
    width: auto;
    padding: 10px 20px;
    font-size: 13px;
  }
</style>
