'use client';

import { Stats } from '@/lib/api';

interface StatsBarProps {
  stats: Stats | null;
  loading?: boolean;
}

export function StatsBar({ stats, loading }: StatsBarProps) {
  if (loading) {
    return (
      <div className="flex gap-4 text-sm text-gray-500">
        <span>Loading stats...</span>
      </div>
    );
  }

  if (!stats) {
    return null;
  }

  return (
    <div className="flex flex-wrap gap-4 text-sm">
      <div className="flex items-center gap-1">
        <span className="text-gray-500">Total:</span>
        <span className="font-semibold text-white">{stats.total_games}</span>
      </div>
      <div className="flex items-center gap-1">
        <span className="text-gray-500">Enriched:</span>
        <span className="font-semibold text-green-400">{stats.enriched_games}</span>
      </div>
      <div className="flex items-center gap-1">
        <span className="text-gray-500">Pending:</span>
        <span className="font-semibold text-yellow-400">{stats.pending_games}</span>
      </div>
    </div>
  );
}
