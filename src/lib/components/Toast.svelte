<script>
  let { message = "", type = "info", onClose } = $props();
  const isError = $derived(type === "error");
  const isSuccess = $derived(type === "success");
</script>

{#if message}
  <div class="toast" class:error={isError} class:success={isSuccess} role="status">
    {message}
    <button class="close" onclick={onClose} aria-label="Dismiss">&times;</button>
  </div>
{/if}

<style>
  .toast {
    position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%);
    background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--radius); padding: 10px 36px 10px 14px;
    font-size: 13px; z-index: 200; box-shadow: var(--shadow); max-width: 90vw;
  }
  .toast.error { border-color: var(--err); color: var(--err); }
  .toast.success { border-color: var(--ok); color: var(--ok); }
  .close {
    position: absolute; right: 8px; top: 50%; transform: translateY(-50%);
    border: none; background: none; cursor: pointer; color: inherit; opacity: 0.5;
    font-size: 16px; line-height: 1;
  }
</style>
