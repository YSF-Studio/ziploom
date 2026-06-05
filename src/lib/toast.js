/** @typedef {'success' | 'error' | 'info'} ToastType */

/**
 * @param {(msg: string, type?: ToastType) => void} show
 * @param {string} message
 * @param {ToastType} [type='info']
 */
export function notify(show, message, type = "info") {
  show?.(message, type);
}
