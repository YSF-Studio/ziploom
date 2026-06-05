<script>
  import { setTheme } from "../theme.js";

  let { value = "light", onChange } = $props();

  const options = [
    { id: "light", label: "Light mode" },
    { id: "dark", label: "Dark mode" },
    { id: "system", label: "System default" },
  ];

  function pick(id) {
    setTheme(id);
    onChange?.(id);
  }
</script>

<div class="themes" role="radiogroup" aria-label="Theme">
  {#each options as opt}
    <button
      class:active={value === opt.id}
      role="radio"
      aria-checked={value === opt.id}
      aria-label={opt.label}
      title={opt.label}
      onclick={() => pick(opt.id)}
    >{opt.label}</button>
  {/each}
</div>

<style>
  .themes { display: flex; gap: 8px; flex-wrap: wrap; }
  .themes button {
    padding: 8px 14px; border-radius: 20px; border: 1px solid var(--border);
    background: var(--surface); color: var(--muted); cursor: pointer; font-size: 13px;
  }
  .themes button.active {
    background: var(--accent); border-color: var(--accent); color: #fff;
  }
</style>
