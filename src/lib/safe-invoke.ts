import { invoke } from '@tauri-apps/api/core';

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export class IpcError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'IpcError';
  }
}

export async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new IpcError('Tauri IPC not available');
  }
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw new IpcError(typeof e === 'string' ? e : String(e));
  }
}
