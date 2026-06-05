<script>
  let { size = 28, alt = "ZipLoom" } = $props();
</script>

<img src="/logo.png" {alt} width={size} height={size} class="logo" />

<style>
  .logo { border-radius: 6px; flex-shrink: 0; display: block; object-fit: cover; }
</style>
