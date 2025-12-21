'use client';

import { useState, useEffect, useCallback } from 'react';
import { GameCard } from '@/components/GameCard';
import { SearchBar } from '@/components/SearchBar';
import { StatsBar } from '@/components/StatsBar';
import { EnrichModal } from '@/components/EnrichModal';
import { EditModal } from '@/components/EditModal';
import { AdjustMatchModal } from '@/components/AdjustMatchModal';
import { SettingsModal } from '@/components/SettingsModal';
import { Game, GameDetail, Stats, getGames, getGame, searchGames, scanGames, getStats } from '@/lib/api';

export default function Home() {
  const [games, setGames] = useState<Game[]>([]);
  const [stats, setStats] = useState<Stats | null>(null);
  const [loading, setLoading] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [enrichModalOpen, setEnrichModalOpen] = useState(false);
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [adjustMatchModalOpen, setAdjustMatchModalOpen] = useState(false);
  const [settingsModalOpen, setSettingsModalOpen] = useState(false);
  const [selectedGame, setSelectedGame] = useState<GameDetail | null>(null);

  const loadGames = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getGames();
      setGames(data);
      setError(null);
    } catch (err) {
      setError('Failed to load games. Is the backend running?');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadStats = useCallback(async () => {
    try {
      const data = await getStats();
      setStats(data);
    } catch (err) {
      console.error('Failed to load stats:', err);
    }
  }, []);

  useEffect(() => {
    loadGames();
    loadStats();
  }, [loadGames, loadStats]);

  const handleSearch = useCallback(async (query: string) => {
    setSearchQuery(query);
    if (!query.trim()) {
      loadGames();
      return;
    }

    try {
      setLoading(true);
      const data = await searchGames(query);
      setGames(data);
      setError(null);
    } catch (err) {
      setError('Search failed');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, [loadGames]);

  const handleScan = async () => {
    try {
      setScanning(true);
      const result = await scanGames();
      alert(`Scan complete: ${result.total_found} games found, ${result.added_or_updated} added/updated`);
      loadGames();
      loadStats();
    } catch (err) {
      alert('Scan failed. Check console for details.');
      console.error(err);
    } finally {
      setScanning(false);
    }
  };

  const handleEnrichComplete = () => {
    loadGames();
    loadStats();
  };

  const handleEditGame = async (gameId: number) => {
    try {
      const gameDetail = await getGame(gameId);
      setSelectedGame(gameDetail);
      setEditModalOpen(true);
    } catch (err) {
      console.error('Failed to load game details:', err);
      alert('Failed to load game details');
    }
  };

  const handleEditSave = (updatedGame: GameDetail) => {
    // Update the game in the local list
    setGames(prevGames =>
      prevGames.map(g =>
        g.id === updatedGame.id
          ? {
              ...g,
              title: updatedGame.title,
              genres: updatedGame.genres ? JSON.parse(updatedGame.genres) : null,
              review_score: updatedGame.review_score,
              review_summary: updatedGame.review_summary,
            }
          : g
      )
    );
    loadStats();
  };

  const handleAdjustMatch = async (gameId: number) => {
    try {
      const gameDetail = await getGame(gameId);
      setSelectedGame(gameDetail);
      setAdjustMatchModalOpen(true);
    } catch (err) {
      console.error('Failed to load game details:', err);
      alert('Failed to load game details');
    }
  };

  const handleAdjustMatchSave = (updatedGame: GameDetail) => {
    // Refresh the game list to show new cover and data
    loadGames();
    loadStats();
  };

  return (
    <main className="min-h-screen p-4 md:p-8">
      {/* Header */}
      <header className="mb-8">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4 mb-6">
          <div>
            <h1 className="text-3xl font-bold text-white flex items-center gap-2">
              <svg className="w-8 h-8 text-gv-accent" fill="currentColor" viewBox="0 0 24 24">
                <path d="M21 6H3c-1.1 0-2 .9-2 2v8c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2zm-10 7H8v3H6v-3H3v-2h3V8h2v3h3v2zm4.5 2c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zm4-3c-.83 0-1.5-.67-1.5-1.5S18.67 9 19.5 9s1.5.67 1.5 1.5-.67 1.5-1.5 1.5z"/>
              </svg>
              GameVault
            </h1>
            <p className="text-gray-500 mt-1">Your personal game library</p>
          </div>

          <div className="flex gap-2">
            <button
              onClick={handleScan}
              disabled={scanning}
              className="px-4 py-2 bg-gv-accent hover:bg-gv-accent/80 disabled:bg-gv-accent/50 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
            >
              {scanning ? (
                <>
                  <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
                  </svg>
                  Scanning...
                </>
              ) : (
                <>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  Scan
                </>
              )}
            </button>

            <button
              onClick={() => setEnrichModalOpen(true)}
              className="px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
              </svg>
              Enrich
            </button>

            <button
              onClick={() => setSettingsModalOpen(true)}
              className="p-2 text-gray-400 hover:text-white hover:bg-gv-hover rounded-lg transition-colors"
              title="Settings"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
          </div>
        </div>

        <div className="flex flex-col md:flex-row md:items-center gap-4">
          <div className="flex-1 max-w-md">
            <SearchBar onSearch={handleSearch} />
          </div>
          <StatsBar stats={stats} loading={loading} />
        </div>
      </header>

      {/* Error State */}
      {error && (
        <div className="mb-6 p-4 bg-red-900/50 border border-red-500 rounded-lg text-red-200">
          {error}
        </div>
      )}

      {/* Loading State */}
      {loading && (
        <div className="flex items-center justify-center py-20">
          <div className="flex items-center gap-3 text-gray-400">
            <svg className="animate-spin w-6 h-6" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
            </svg>
            Loading games...
          </div>
        </div>
      )}

      {/* Empty State */}
      {!loading && games.length === 0 && (
        <div className="flex flex-col items-center justify-center py-20 text-gray-500">
          <svg className="w-16 h-16 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p className="text-lg mb-2">No games found</p>
          <p className="text-sm">Click &quot;Scan&quot; to scan your games folder</p>
        </div>
      )}

      {/* Game Grid */}
      {!loading && games.length > 0 && (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-4">
          {games.map((game) => (
            <GameCard
              key={game.id}
              game={game}
              onEdit={() => handleEditGame(game.id)}
              onAdjustMatch={() => handleAdjustMatch(game.id)}
            />
          ))}
        </div>
      )}

      {/* Enrich Modal */}
      <EnrichModal
        isOpen={enrichModalOpen}
        onClose={() => setEnrichModalOpen(false)}
        onComplete={handleEnrichComplete}
      />

      {/* Edit Modal */}
      <EditModal
        isOpen={editModalOpen}
        game={selectedGame}
        onClose={() => {
          setEditModalOpen(false);
          setSelectedGame(null);
        }}
        onSave={handleEditSave}
      />

      {/* Adjust Match Modal */}
      <AdjustMatchModal
        isOpen={adjustMatchModalOpen}
        game={selectedGame}
        onClose={() => {
          setAdjustMatchModalOpen(false);
          setSelectedGame(null);
        }}
        onSave={handleAdjustMatchSave}
      />

      {/* Settings Modal */}
      <SettingsModal
        isOpen={settingsModalOpen}
        onClose={() => setSettingsModalOpen(false)}
      />
    </main>
  );
}
