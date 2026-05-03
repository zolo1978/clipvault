import { safeInvoke, IpcError } from '../lib/safe-invoke';

// [CODEX_FALLBACK] Frontend implementation — Codex skill unavailable

export type ContentType = 'text' | 'image' | 'file_path';

export interface ClipSummary {
  id: string;
  content_type: ContentType;
  preview: string;
  is_favorite: boolean;
  is_sensitive: boolean;
  created_at: number;
}

export interface Clip {
  id: string;
  content_type: ContentType;
  content: string; // base64 encoded
  preview: string;
  content_hash: string;
  is_favorite: boolean;
  is_sensitive: boolean;
  created_at: number;
}

export interface MonitorStatus {
  is_running: boolean;
  clips_captured: number;
}

export interface AppConfig {
  max_clips: number;
  keep_days: number;
  monitor_interval_ms: number;
  exclude_sources: string[];
  shortcut: string;
  sensitive_detection_enabled: boolean;
}

export async function listClips(params: {
  limit: number;
  offset: number;
  contentType?: ContentType;
}): Promise<ClipSummary[]> {
  return safeInvoke<ClipSummary[]>('list_clips', params);
}

export async function searchClips(params: {
  query: string;
  contentType?: ContentType;
  limit: number;
}): Promise<ClipSummary[]> {
  return safeInvoke<ClipSummary[]>('search_clips', params);
}

export async function getClip(id: string): Promise<Clip> {
  return safeInvoke<Clip>('get_clip', { id });
}

export async function deleteClip(id: string): Promise<void> {
  return safeInvoke<void>('delete_clip', { id });
}

export async function deleteClips(ids: string[]): Promise<void> {
  return safeInvoke<void>('delete_clips', { ids });
}

export async function toggleFavorite(id: string): Promise<ClipSummary> {
  return safeInvoke<ClipSummary>('toggle_favorite', { id });
}

export async function purgeClips(params: {
  keepDays?: number;
  keepCount?: number;
}): Promise<number> {
  return safeInvoke<number>('purge_clips', params);
}

export async function pasteClip(id: string): Promise<void> {
  return safeInvoke<void>('paste_clip', { id });
}

export async function viewImageClip(id: string): Promise<void> {
  return safeInvoke<void>('view_image_clip', { id });
}

export async function revealPath(id: string): Promise<void> {
  return safeInvoke<void>('reveal_path', { id });
}

export async function startMonitor(): Promise<void> {
  return safeInvoke<void>('start_monitor');
}

export async function stopMonitor(): Promise<void> {
  return safeInvoke<void>('stop_monitor');
}

export async function getMonitorStatus(): Promise<MonitorStatus> {
  return safeInvoke<MonitorStatus>('monitor_status');
}

export async function getAppConfig(): Promise<AppConfig> {
  return safeInvoke<AppConfig>('get_config');
}

export async function updateAppConfig(config: AppConfig): Promise<AppConfig> {
  return safeInvoke<AppConfig>('update_config', { config });
}

export async function snipScreen(): Promise<ClipSummary> {
  return safeInvoke<ClipSummary>('snip_screen');
}

export async function minimizeWindow(): Promise<void> {
  return safeInvoke<void>('minimize_window');
}

export async function toggleMaximize(): Promise<void> {
  return safeInvoke<void>('toggle_maximize');
}

export async function closeWindow(): Promise<void> {
  return safeInvoke<void>('close_window');
}

export async function quitApp(): Promise<void> {
  return safeInvoke<void>('quit_app');
}

export async function startDrag(): Promise<void> {
  return safeInvoke<void>('start_drag');
}

export async function getSensitiveClipContent(id: string): Promise<string> {
  return safeInvoke<string>('get_sensitive_clip_content', { id });
}

export async function checkSensitiveExpired(id: string): Promise<boolean> {
  return safeInvoke<boolean>('check_sensitive_expired', { id });
}

export { IpcError };
