/** Poll backend progress while a long-running Tauri command is in flight. */
export function startProgressPoll(invoke, onUpdate, intervalMs = 200) {
  let active = true;
  const tick = async () => {
    if (!active) return;
    try {
      const p = await invoke("get_progress");
      onUpdate(p);
    } catch {
      /* ignore — command may not be registered in browser-only tests */
    }
  };
  tick();
  const timer = setInterval(tick, intervalMs);
  return () => {
    active = false;
    clearInterval(timer);
  };
}

/** Run an async operation with live progress updates. */
export async function withProgress(invoke, operation, onUpdate) {
  const stop = startProgressPoll(invoke, onUpdate);
  try {
    return await operation();
  } finally {
    stop();
    try {
      onUpdate(await invoke("get_progress"));
    } catch {
      /* ignore */
    }
  }
}
