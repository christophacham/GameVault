const API_URL = process.env.NEXT_PUBLIC_API_URL || '';

export interface Game {
  id: number;
  title: string;
  cover_url: string | null;
  genres: string[] | null;
  review_score: number | null;
  review_summary: string | null;
  match_status: string;
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
  genres: string | null;
  developers: string | null;
  publishers: string | null;
  review_score: number | null;
  review_count: number | null;
  review_summary: string | null;
  size_bytes: number | null;
  match_confidence: number | null;
  match_status: string;
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

export async function scanGames(): Promise<ScanResult> {
  return fetchApi<ScanResult>('/scan', { method: 'POST' });
}

export async function enrichGames(): Promise<EnrichResult> {
  return fetchApi<EnrichResult>('/enrich', { method: 'POST' });
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
