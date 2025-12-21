const API_URL = process.env.NEXT_PUBLIC_API_URL || '';

export interface Game {
  id: number;
  title: string;
  cover_url: string | null;
  local_cover_path: string | null;
  genres: string[] | null;
  review_score: number | null;
  review_summary: string | null;
  match_status: string;
  user_status: string | null;
  hltb_main_mins: number | null;
}

export interface GameDetail {
  id: number;
  folder_path: string;
  folder_name: string;
  title: string;
  igdb_id: number | null;
  steam_app_id: number | null;
  summary: string | null;
  release_date: string | null;
  cover_url: string | null;
  background_url: string | null;
  local_cover_path: string | null;
  local_background_path: string | null;
  genres: string | null;
  developers: string | null;
  publishers: string | null;
  review_score: number | null;
  review_count: number | null;
  review_summary: string | null;
  size_bytes: number | null;
  match_confidence: number | null;
  match_status: string;
  user_status: string | null;
  playtime_mins: number | null;
  hltb_main_mins: number | null;
  hltb_extra_mins: number | null;
  hltb_completionist_mins: number | null;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export interface Stats {
  total_games: number;
  matched_games: number;
  pending_games: number;
  enriched_games: number;
}

export interface ScanResult {
  total_found: number;
  added_or_updated: number;
}

export interface EnrichResult {
  enriched: number;
  failed: number;
  remaining: number;
  total: number;
}

export interface ExportResult {
  exported: number;
  skipped: number;
  failed: number;
  total: number;
}

export interface ImportResult {
  imported: number;
  skipped: number;
  not_found: number;
  failed: number;
  total: number;
}

async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}/api${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!res.ok) {
    throw new Error(`API error: ${res.status}`);
  }

  const json: ApiResponse<T> = await res.json();

  if (!json.success) {
    throw new Error(json.error || 'Unknown error');
  }

  return json.data as T;
}

export async function getGames(): Promise<Game[]> {
  return fetchApi<Game[]>('/games');
}

export async function getGame(id: number): Promise<GameDetail> {
  return fetchApi<GameDetail>(`/games/${id}`);
}

export async function searchGames(query: string): Promise<Game[]> {
  return fetchApi<Game[]>(`/games/search?q=${encodeURIComponent(query)}`);
}

export async function getRecentGames(): Promise<Game[]> {
  return fetchApi<Game[]>('/games/recent');
}

export async function scanGames(): Promise<ScanResult> {
  return fetchApi<ScanResult>('/scan', { method: 'POST' });
}

export async function enrichGames(): Promise<EnrichResult> {
  return fetchApi<EnrichResult>('/enrich', { method: 'POST' });
}

export async function exportGames(): Promise<ExportResult> {
  return fetchApi<ExportResult>('/export', { method: 'POST' });
}

export async function importGames(): Promise<ImportResult> {
  return fetchApi<ImportResult>('/import', { method: 'POST' });
}

export interface UpdateGameRequest {
  title?: string;
  summary?: string;
  genres?: string[];
  developers?: string[];
  publishers?: string[];
  release_date?: string;
  review_score?: number;
}

export async function updateGame(id: number, data: UpdateGameRequest): Promise<GameDetail> {
  return fetchApi<GameDetail>(`/games/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export interface RematchResult {
  steam_app_id: number;
  title: string;
  summary: string | null;
  genres: string[] | null;
  developers: string[] | null;
  publishers: string[] | null;
  release_date: string | null;
  cover_url: string | null;
  review_score: number | null;
  review_summary: string | null;
}

export async function previewRematch(id: number, steamInput: string): Promise<RematchResult> {
  return fetchApi<RematchResult>(`/games/${id}/match`, {
    method: 'POST',
    body: JSON.stringify({ steam_input: steamInput }),
  });
}

export async function confirmRematch(id: number, steamInput: string): Promise<GameDetail> {
  return fetchApi<GameDetail>(`/games/${id}/match/confirm`, {
    method: 'POST',
    body: JSON.stringify({ steam_input: steamInput }),
  });
}

export async function getStats(): Promise<Stats> {
  return fetchApi<Stats>('/stats');
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function getReviewColor(score: number | null): string {
  if (score === null) return 'text-gray-500';
  if (score >= 80) return 'text-green-400';
  if (score >= 60) return 'text-yellow-400';
  if (score >= 40) return 'text-orange-400';
  return 'text-red-400';
}

export function getCoverUrl(game: Game | GameDetail): string | null {
  if (game.local_cover_path) {
    return `${API_URL}/api/games/${game.id}/cover`;
  }
  return game.cover_url;
}

export function getBackgroundUrl(game: GameDetail): string | null {
  if (game.local_background_path) {
    return `${API_URL}/api/games/${game.id}/background`;
  }
  return game.background_url;
}

// ============================================================================
// Configuration API
// ============================================================================

export interface ConfigPaths {
  game_library: string;
  cache: string;
  game_library_exists: boolean;
  cache_exists: boolean;
}

export interface ConfigServer {
  port: number;
  auto_open_browser: boolean;
  bind_address: string;
}

export interface Config {
  paths: ConfigPaths;
  server: ConfigServer;
}

export interface ConfigUpdateRequest {
  game_library: string;
  cache: string;
  port: number;
  auto_open_browser: boolean;
}

export interface ConfigUpdateResponse {
  success: boolean;
  restart_required: boolean;
  message: string;
}

export async function getConfig(): Promise<Config> {
  return fetchApi<Config>('/config');
}

export async function updateConfig(data: ConfigUpdateRequest): Promise<ConfigUpdateResponse> {
  return fetchApi<ConfigUpdateResponse>('/config', {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function shutdownServer(): Promise<string> {
  return fetchApi<string>('/shutdown', { method: 'POST' });
}

export async function restartServer(): Promise<string> {
  return fetchApi<string>('/restart', { method: 'POST' });
}

export interface ConfigStatusResponse {
  needs_setup: boolean;
  game_library_configured: boolean;
  game_library_path: string;
}

export async function getConfigStatus(): Promise<ConfigStatusResponse> {
  return fetchApi<ConfigStatusResponse>('/config/status');
}
