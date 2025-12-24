'use client';

import { useState, useEffect } from 'react';
import { GameDetail, RematchResult, previewRematch, confirmRematch, getReviewColor } from '@/lib/api';

interface AdjustMatchModalProps {
  isOpen: boolean;
  game: GameDetail | null;
  onClose: () => void;
  onSave: (updatedGame: GameDetail) => void;
}

export function AdjustMatchModal({ isOpen, game, onClose, onSave }: AdjustMatchModalProps) {
  const [steamInput, setSteamInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [preview, setPreview] = useState<RematchResult | null>(null);
  const [showHelp, setShowHelp] = useState(false);

  // Escape key to close modal
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !loading && !confirming) {
        handleClose();
      }
    };
    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
    }
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, loading, confirming]);

  const handlePreview = async () => {
    if (!game || !steamInput.trim()) return;

    setLoading(true);
    setError(null);
    setPreview(null);

    try {
      const result = await previewRematch(game.id, steamInput.trim());
      setPreview(result);
    } catch (err) {
      console.error('Preview failed:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch Steam data');
    } finally {
      setLoading(false);
    }
  };

  const handleConfirm = async () => {
    if (!game || !preview) return;

    setConfirming(true);
    setError(null);

    try {
      const updatedGame = await confirmRematch(game.id, steamInput.trim());
      onSave(updatedGame);
      handleClose();
    } catch (err) {
      console.error('Confirm failed:', err);
      setError(err instanceof Error ? err.message : 'Failed to update game');
    } finally {
      setConfirming(false);
    }
  };

  const handleClose = () => {
    setSteamInput('');
    setPreview(null);
    setError(null);
    setShowHelp(false);
    onClose();
  };

  if (!isOpen || !game) return null;

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
      <div className="bg-gv-card rounded-xl w-full max-w-2xl shadow-2xl max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gv-hover">
          <h2 className="text-xl font-bold text-white">Adjust Match</h2>
          <button
            onClick={handleClose}
            disabled={loading || confirming}
            className="text-gray-400 hover:text-white disabled:opacity-50"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          {/* Current Game */}
          <div className="text-sm text-gray-400">
            Currently matching: <span className="text-white font-medium">{game.title}</span>
          </div>

          {/* Error Message */}
          {error && (
            <div className="p-3 rounded-lg bg-red-900/50 text-red-200">
              {error}
            </div>
          )}

          {/* Input Section */}
          <div>
            <div className="flex items-center gap-2 mb-2">
              <label className="block text-sm font-medium text-gray-300">
                Steam URL or App ID
              </label>
              <button
                onClick={() => setShowHelp(!showHelp)}
                className="text-gray-500 hover:text-gv-accent transition-colors"
                title="Show help"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </button>
            </div>

            {/* Help Tooltip */}
            {showHelp && (
              <div className="mb-3 p-4 bg-gv-hover rounded-lg text-sm text-gray-300">
                <p className="font-medium text-white mb-2">How to find the correct game:</p>
                <ol className="list-decimal list-inside space-y-1">
                  <li>Go to <a href="https://store.steampowered.com" target="_blank" rel="noopener noreferrer" className="text-gv-accent hover:underline">store.steampowered.com</a></li>
                  <li>Search for your game</li>
                  <li>Copy the URL from your browser</li>
                  <li>Paste it here, or just enter the number</li>
                </ol>
                <p className="mt-2 text-gray-400">
                  Example: <code className="bg-black/30 px-1 rounded">https://store.steampowered.com/app/292030/</code> or just <code className="bg-black/30 px-1 rounded">292030</code>
                </p>
              </div>
            )}

            <div className="flex gap-2">
              <input
                type="text"
                value={steamInput}
                onChange={(e) => setSteamInput(e.target.value)}
                placeholder="https://store.steampowered.com/app/292030/ or 292030"
                className="flex-1 px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent"
                disabled={loading || confirming}
              />
              <button
                onClick={handlePreview}
                disabled={loading || confirming || !steamInput.trim()}
                className="px-4 py-2 bg-gv-accent hover:bg-gv-accent/80 disabled:bg-gv-accent/50 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
              >
                {loading ? (
                  <>
                    <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
                    </svg>
                    Loading...
                  </>
                ) : (
                  'Preview'
                )}
              </button>
            </div>
          </div>

          {/* Preview Card */}
          {preview && (
            <div className="border border-gv-hover rounded-lg overflow-hidden">
              <div className="p-4 bg-gv-hover/50">
                <h3 className="text-lg font-semibold text-white mb-1">Preview: Match Found</h3>
                <p className="text-sm text-gray-400">Steam App ID: {preview.steam_app_id}</p>
              </div>

              <div className="p-4 flex gap-4">
                {/* Cover Image */}
                {preview.cover_url && (
                  <div className="flex-shrink-0">
                    <img
                      src={preview.cover_url}
                      alt={preview.title}
                      className="w-32 h-auto rounded"
                    />
                  </div>
                )}

                {/* Details */}
                <div className="flex-1 space-y-2">
                  <h4 className="font-semibold text-white text-lg">{preview.title}</h4>

                  {preview.genres && preview.genres.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {preview.genres.slice(0, 5).map((genre, i) => (
                        <span key={i} className="text-xs px-2 py-0.5 bg-gv-hover rounded text-gray-300">
                          {genre}
                        </span>
                      ))}
                    </div>
                  )}

                  {preview.developers && preview.developers.length > 0 && (
                    <p className="text-sm text-gray-400">
                      Developer: <span className="text-gray-300">{preview.developers.join(', ')}</span>
                    </p>
                  )}

                  {preview.release_date && (
                    <p className="text-sm text-gray-400">
                      Released: <span className="text-gray-300">{preview.release_date}</span>
                    </p>
                  )}

                  {preview.review_score !== null && (
                    <p className={`text-sm ${getReviewColor(preview.review_score)}`}>
                      {preview.review_score}% - {preview.review_summary || 'No reviews'}
                    </p>
                  )}

                  {preview.summary && (
                    <p className="text-sm text-gray-400 line-clamp-3">
                      {preview.summary}
                    </p>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 p-6 border-t border-gv-hover">
          <button
            onClick={handleClose}
            disabled={loading || confirming}
            className="px-4 py-2 text-gray-400 hover:text-white disabled:opacity-50"
          >
            Cancel
          </button>
          {preview && (
            <button
              onClick={handleConfirm}
              disabled={confirming}
              className="px-4 py-2 bg-green-600 hover:bg-green-500 disabled:bg-green-600/50 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
            >
              {confirming ? (
                <>
                  <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
                  </svg>
                  Applying...
                </>
              ) : (
                <>
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                  Confirm Match
                </>
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
