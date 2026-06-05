export function idle() {
  return { status: "idle", message: "" };
}

export function running(message = "Processing...") {
  return { status: "running", message, progress: 0, detail: "" };
}

export function success(message, data = {}) {
  return { status: "success", message, ...data };
}

export function error(message) {
  return { status: "error", message };
}
