/** Prompt for archive password only when needed. Returns null if cancelled. */
export function promptPassword(onOpen) {
  return new Promise((resolve) => {
    onOpen({
      resolve: (password) => resolve(password || null),
      cancel: () => resolve(null),
    });
  });
}

function isPasswordError(msg) {
  const m = String(msg);
  return (
    m.includes("PASSWORD_NEEDED") ||
    m.includes("WRONG_PASSWORD") ||
    /password/i.test(m) ||
    /decrypt/i.test(m)
  );
}

/**
 * Run an archive operation, prompting for password when needed.
 * Probes archive_needs_password first, then retries on wrong/missing password.
 */
export async function withArchivePassword(path, invoke, onPrompt, run) {
  let password = null;

  try {
    const needs = await invoke("archive_needs_password", { path });
    if (needs) {
      password = await onPrompt();
      if (!password) throw new Error("Password required");
    }
  } catch (probeErr) {
    const probeMsg = String(probeErr);
    if (isPasswordError(probeMsg)) {
      password = await onPrompt();
      if (!password) throw new Error("Password required");
    }
  }

  const maxAttempts = 5;
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      return await run(password);
    } catch (e) {
      const msg = String(e);
      if (!isPasswordError(msg) || attempt >= maxAttempts - 1) throw e;
      password = await onPrompt();
      if (!password) throw new Error("Password required");
    }
  }
}
