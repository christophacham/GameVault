'use client';

import { useState, useEffect } from 'react';
import { GameDetail, updateGame, UpdateGameRequest } from '@/lib/api';

interface EditModalProps {
  isOpen: boolean;
  game: GameDetail | null;
  onClose: () => void;
  onSave: (updatedGame: GameDetail) => void;
}

export function EditModal({ isOpen, game, onClose, onSave }: EditModalProps) {
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form state
  const [title, setTitle] = useState('');
  const [summary, setSummary] = useState('');
  const [genres, setGenres] = useState('');
  const [developers, setDevelopers] = useState('');
  const [publishers, setPublishers] = useState('');
  const [releaseDate, setReleaseDate] = useState('');
  const [reviewScore, setReviewScore] = useState('');
  const [dateError, setDateError] = useState<string | null>(null);

  // Escape key to close modal
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !saving) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
    }
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, saving, onClose]);

  // Reset form when game changes
  useEffect(() => {
    if (game) {
      setTitle(game.title || '');
      setSummary(game.summary || '');
      // Parse JSON arrays to comma-separated strings
      setGenres(parseJsonArray(game.genres));
      setDevelopers(parseJsonArray(game.developers));
      setPublishers(parseJsonArray(game.publishers));
      setReleaseDate(game.release_date || '');
      setReviewScore(game.review_score?.toString() || '');
    }
  }, [game]);

  const parseJsonArray = (jsonStr: string | null): string => {
    if (!jsonStr) return '';
    try {
      const arr = JSON.parse(jsonStr);
      return Array.isArray(arr) ? arr.join(', ') : '';
    } catch {
      return '';
    }
  };

  const parseCommaSeparated = (str: string): string[] => {
    return str.split(',').map(s => s.trim()).filter(s => s.length > 0);
  };

  // Validate date format (YYYY-MM-DD or empty)
  const validateDate = (date: string): boolean => {
    if (!date) return true; // Empty is valid
    const dateRegex = /^\d{4}-\d{2}-\d{2}$/;
    if (!dateRegex.test(date)) return false;
    // Check if it's a valid date
    const parsed = new Date(date);
    return !isNaN(parsed.getTime());
  };

  const handleDateChange = (value: string) => {
    setReleaseDate(value);
    if (value && !validateDate(value)) {
      setDateError('Use format YYYY-MM-DD');
    } else {
      setDateError(null);
    }
  };

  // Check if form is valid for submission
  const isFormValid = title.trim().length > 0 && !dateError;

  const handleSave = async () => {
    if (!game) return;

    setSaving(true);
    setError(null);

    try {
      const data: UpdateGameRequest = {};

      // Only include changed fields
      if (title !== game.title) data.title = title;
      if (summary !== (game.summary || '')) data.summary = summary;
      if (genres !== parseJsonArray(game.genres)) {
        data.genres = parseCommaSeparated(genres);
      }
      if (developers !== parseJsonArray(game.developers)) {
        data.developers = parseCommaSeparated(developers);
      }
      if (publishers !== parseJsonArray(game.publishers)) {
        data.publishers = parseCommaSeparated(publishers);
      }
      if (releaseDate !== (game.release_date || '')) data.release_date = releaseDate;
      if (reviewScore !== (game.review_score?.toString() || '')) {
        const score = parseInt(reviewScore);
        if (!isNaN(score) && score >= 0 && score <= 100) {
          data.review_score = score;
        }
      }

      const updatedGame = await updateGame(game.id, data);
      onSave(updatedGame);
      onClose();
    } catch (err) {
      console.error('Failed to update game:', err);
      setError('Failed to save changes. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen || !game) return null;

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
      <div className="bg-gv-card rounded-xl w-full max-w-2xl shadow-2xl max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gv-hover">
          <h2 className="text-xl font-bold text-white">Edit Game Details</h2>
          <button
            onClick={onClose}
            disabled={saving}
            className="text-gray-400 hover:text-white disabled:opacity-50"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Form */}
        <div className="p-6 space-y-4">
          {/* Error Message */}
          {error && (
            <div className="p-3 rounded-lg bg-red-900/50 text-red-200">
              {error}
            </div>
          )}

          {/* Title */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Title <span className="text-red-400">*</span>
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className={`w-full px-3 py-2 bg-gv-hover border rounded-lg text-white focus:outline-none ${!title.trim() ? 'border-red-500/50' : 'border-gv-hover focus:border-gv-accent'}`}
            />
            {!title.trim() && <p className="text-red-400 text-xs mt-1">Title is required</p>}
          </div>

          {/* Genres */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Genres <span className="text-gray-500">(comma-separated)</span>
            </label>
            <input
              type="text"
              value={genres}
              onChange={(e) => setGenres(e.target.value)}
              placeholder="Action, RPG, Adventure"
              className="w-full px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent"
            />
          </div>

          {/* Developers */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Developers <span className="text-gray-500">(comma-separated)</span>
            </label>
            <input
              type="text"
              value={developers}
              onChange={(e) => setDevelopers(e.target.value)}
              className="w-full px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent"
            />
          </div>

          {/* Publishers */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Publishers <span className="text-gray-500">(comma-separated)</span>
            </label>
            <input
              type="text"
              value={publishers}
              onChange={(e) => setPublishers(e.target.value)}
              className="w-full px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent"
            />
          </div>

          {/* Release Date & Review Score Row */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Release Date</label>
              <input
                type="text"
                value={releaseDate}
                onChange={(e) => handleDateChange(e.target.value)}
                placeholder="YYYY-MM-DD"
                className={`w-full px-3 py-2 bg-gv-hover border rounded-lg text-white focus:outline-none ${dateError ? 'border-red-500' : 'border-gv-hover focus:border-gv-accent'}`}
              />
              {dateError && <p className="text-red-400 text-xs mt-1">{dateError}</p>}
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                Review Score <span className="text-gray-500">(0-100)</span>
              </label>
              <input
                type="number"
                min="0"
                max="100"
                value={reviewScore}
                onChange={(e) => setReviewScore(e.target.value)}
                className="w-full px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent"
              />
            </div>
          </div>

          {/* Summary */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">Summary</label>
            <textarea
              value={summary}
              onChange={(e) => setSummary(e.target.value)}
              rows={4}
              className="w-full px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent resize-none"
            />
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 p-6 border-t border-gv-hover">
          <button
            onClick={onClose}
            disabled={saving}
            className="px-4 py-2 text-gray-400 hover:text-white disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={saving || !isFormValid}
            title={!isFormValid ? 'Title is required and date must be valid' : ''}
            className="px-4 py-2 bg-gv-accent hover:bg-gv-accent/80 disabled:bg-gv-accent/50 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors flex items-center gap-2"
          >
            {saving && (
              <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
              </svg>
            )}
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
}
