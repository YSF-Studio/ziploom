<script>
  export let size = 22;
</script>

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width={size} height={size} class="logo-svg">
  <defs><linearGradient id="a"><stop offset="0%" stop-color="#12243c"/><stop offset="100%" stop-color="#0d1a2e"/></linearGradient></defs>
  <rect x="8" y="8" width="496" height="496" rx="64" fill="url(#a)" stroke="#1a4066" stroke-width="3"/>
  <g transform="translate(148,55)">
    <path d="M10 22h28l70 5 70-5h28l10 23L108 400 0 45Z" fill="#080c12" stroke="#1a80ff" stroke-width="3.5"/>
    <g fill="#080c12" stroke="#1a80ff" stroke-width="3">
      <polygon points="30,35 186,35 186,55 110,55 110,70 186,70 186,50 30,50 30,70 90,70 90,50 30,50"/>
      <polygon points="30,55 110,340 186,340 106,55"/>
      <polygon points="30,320 186,320 186,335 110,335 110,320 186,320 186,345 30,345 30,325 90,325 90,340 30,340"/>
    </g>
    <g fill="#080c12" stroke="#1a80ff" stroke-width="2.5">
      <rect x="99" y="35" width="18" height="330" rx="2"/>
      <rect x="80" y="280" width="56" height="16" rx="2"/>
      <rect x="92" y="350" width="32" height="20" rx="2"/>
      <line x1="108" y1="350" x2="108" y2="370" stroke="#1a80ff" stroke-width="2"/>
      <line x1="92" y1="360" x2="124" y2="360" stroke="#1a80ff" stroke-width="2"/>
    </g>
    <rect x="40" y="270" width="20" height="20" fill="#080c12" stroke="#1a80ff" stroke-width="2"/>
    <rect x="156" y="270" width="20" height="20" fill="#080c12" stroke="#1a80ff" stroke-width="2"/>
    <rect x="94" y="24" width="28" height="8" fill="#080c12" stroke="#1a80ff" stroke-width="2"/>
    <g stroke="#1a80ff" stroke-width="2" fill="#080c12">
      <rect x="98" y="385" width="20" height="8"/>
      <rect x="104" y="379" width="8" height="20"/>
    </g>
  </g>
</svg>

<style>
  .logo-svg { border-radius: 4px; flex-shrink: 0; }
</style>
