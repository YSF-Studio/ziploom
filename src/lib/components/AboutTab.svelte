<script>
  import Logo from "../Logo.svelte";

  const features = [
    "8 formats: ZIP · TAR · GZ · BZ2 · XZ · Zstandard — compress & extract",
    "7z & RAR — pure Rust extract (sevenz-rust, unrar crate)",
    "Password-protected ZIP (AES-256, 7-Zip / WinRAR compatible)",
    "Zstandard (.zst/.tzst) — modern fast compression",
    "Forensic analysis — magic byte verification",
    "Entropy scanning & anomaly detection",
    "Batch hashing (MD5, SHA-1, SHA-256)",
    "100% offline · zero data collection",
  ];
</script>

<div class="about">
  <div class="about-hero">
    <Logo size={72} />
  </div>
  <h1>ZipLoom</h1>
  <p class="ver">Version 1.0</p>
  <p class="tag">Archive Utility &amp; Forensic Inspector</p>
  <p class="sub">Pure Rust · Offline · Private</p>

  <div class="features">
    {#each features as f}
      <div class="feat">
        <span class="dot" aria-hidden="true"></span>
        <span>{f}</span>
      </div>
    {/each}
  </div>

  <div class="disclaimer">
    <strong>LEGAL DISCLAIMER</strong>
    <p>
      ZipLoom is provided "AS-IS" with NO WARRANTY of any kind, express or implied.
      NOT certified for court evidence, ISO auditing, NIST validation, or any forensic standard compliance.
      All forensic analysis results are informational only. Users must independently verify all findings before
      use in any legal, compliance, or security context.
      This tool is intended for authorized security research and personal archive management only.
    </p>
  </div>

  <p class="foot">Made by Yusuf Shalahuddin Al Ayyubi As Sobari</p>
  <p class="foot brand">YSF Studio</p>
  <a class="link-btn" href="https://ysfloom.com" target="_blank" rel="noopener noreferrer">ysfloom.com</a>
  <p class="copy">© 2026 YSF Studio — All rights reserved</p>
</div>
