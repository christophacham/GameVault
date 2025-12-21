'use client';

import { useState, useEffect } from 'react';
import { Config, getConfig, updateConfig, shutdownServer, restartServer } from '@/lib/api';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [config, setConfig] = useState<Config | null>(null);

  // Form state
  const [gameLibrary, setGameLibrary] = useState('');
  const [cachePath, setCachePath] = useState('');
  const [port, setPort] = useState(3000);
  const [autoOpenBrowser, setAutoOpenBrowser] = useState(true);

  // Track validation status
  const [gameLibraryExists, setGameLibraryExists] = useState(true);

  // Load config when modal opens
  useEffect(() => {
    if (isOpen) {
      loadConfig();
    }
  }, [isOpen]);

  const loadConfig = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await getConfig();
      setConfig(data);
      setGameLibrary(data.paths.game_library);
      setCachePath(data.paths.cache);
      setPort(data.server.port);
      setAutoOpenBrowser(data.server.auto_open_browser);
      setGameLibraryExists(data.paths.game_library_exists);
    } catch (err) {
      console.error('Failed to load config:', err);
      setError('Failed to load configuration');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);

    try {
      const result = await updateConfig({
        game_library: gameLibrary,
        cache: cachePath,
        port,
        auto_open_browser: autoOpenBrowser,
      });

      if (result.restart_required) {
        alert('Settings saved. Restart GameVault for port changes to take effect.');
      }

      onClose();
    } catch (err) {
      console.error('Failed to save config:', err);
      setError(err instanceof Error ? err.message : 'Failed to save configuration');
    } finally {
      setSaving(false);
    }
  };

  const handleShutdown = async () => {
    if (confirm('Are you sure you want to shutdown GameVault?')) {
      try {
        await shutdownServer();
        // Server will shutdown, page will become unresponsive
      } catch {
        // Expected - server shuts down
      }
    }
  };

  const handleRestart = async () => {
    if (confirm('Restart GameVault? The page will reload automatically.')) {
      try {
        await restartServer();
        // Wait a moment then reload the page
        setTimeout(() => {
          window.location.reload();
        }, 2000);
      } catch {
        // Expected - server restarts
        setTimeout(() => {
          window.location.reload();
        }, 2000);
      }
    }
  };

  // Check if game library needs configuration
  const needsSetup = !gameLibrary || gameLibrary === '.' || gameLibrary.endsWith('\\.' ) || gameLibrary.endsWith('/.');

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4">
      <div className="bg-gv-card rounded-xl w-full max-w-lg shadow-2xl max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gv-hover">
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            Settings
          </h2>
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

        {/* Content */}
        <div className="p-6 space-y-5">
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <svg className="animate-spin w-8 h-8 text-gv-accent" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
              </svg>
            </div>
          ) : (
            <>
              {/* Error Message */}
              {error && (
                <div className="p-3 rounded-lg bg-red-900/50 text-red-200">
                  {error}
                </div>
              )}

              {/* Paths Section */}
              <div className="space-y-4">
                <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wide">Paths</h3>

                {/* Game Library Path */}
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">
                    Game Library Path
                  </label>
                  <input
                    type="text"
                    value={gameLibrary}
                    onChange={(e) => setGameLibrary(e.target.value)}
                    placeholder="D:\Games"
                    className={`w-full px-3 py-2 bg-gv-hover border rounded-lg text-white focus:outline-none focus:border-gv-accent ${
                      needsSetup ? 'border-yellow-500' : 'border-gv-hover'
                    }`}
                  />
                  {needsSetup && (
                    <p className="mt-1 text-sm text-yellow-400 flex items-center gap-1">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                      </svg>
                      Please set your games folder path
                    </p>
                  )}
                  {!needsSetup && !gameLibraryExists && config && gameLibrary === config.paths.game_library && (
                    <p className="mt-1 text-sm text-red-400 flex items-center gap-1">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                      </svg>
                      Path does not exist
                    </p>
                  )}
                </div>

                {/* Cache Directory */}
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">
                    Cache Directory
                  </label>
                  <input
                    type="text"
                    value={cachePath}
                    onChange={(e) => setCachePath(e.target.value)}
                    placeholder="./cache"
                    className="w-full px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent"
                  />
                  <p className="mt-1 text-xs text-gray-500">
                    Cover images are cached here
                  </p>
                </div>
              </div>

              {/* Server Section */}
              <div className="space-y-4 pt-2">
                <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wide">Server</h3>

                {/* Port */}
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-1">
                    Server Port
                  </label>
                  <input
                    type="number"
                    min="1024"
                    max="65535"
                    value={port}
                    onChange={(e) => setPort(parseInt(e.target.value) || 3000)}
                    className="w-full px-3 py-2 bg-gv-hover border border-gv-hover rounded-lg text-white focus:outline-none focus:border-gv-accent"
                  />
                  <p className="mt-1 text-xs text-yellow-500 flex items-center gap-1">
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    Requires restart to take effect
                  </p>
                </div>

                {/* Auto-open browser */}
                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={autoOpenBrowser}
                    onChange={(e) => setAutoOpenBrowser(e.target.checked)}
                    className="w-4 h-4 rounded border-gray-600 bg-gv-hover text-gv-accent focus:ring-gv-accent focus:ring-offset-0"
                  />
                  <span className="text-gray-300">Auto-open browser on startup</span>
                </label>
              </div>

              {/* Actions Section */}
              <div className="space-y-3 pt-4 border-t border-gv-hover">
                <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wide">Actions</h3>
                <div className="flex gap-2">
                  <button
                    onClick={handleRestart}
                    className="flex-1 px-4 py-2 bg-blue-900/30 hover:bg-blue-900/50 text-blue-300 rounded-lg transition-colors flex items-center justify-center gap-2"
                  >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    Restart
                  </button>
                  <button
                    onClick={handleShutdown}
                    className="flex-1 px-4 py-2 bg-red-900/30 hover:bg-red-900/50 text-red-300 rounded-lg transition-colors flex items-center justify-center gap-2"
                  >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                    </svg>
                    Shutdown
                  </button>
                </div>
              </div>
            </>
          )}
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
            disabled={saving || loading}
            className="px-4 py-2 bg-gv-accent hover:bg-gv-accent/80 disabled:bg-gv-accent/50 text-white rounded-lg font-medium transition-colors flex items-center gap-2"
          >
            {saving && (
              <svg className="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"/>
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
              </svg>
            )}
            Save Settings
          </button>
        </div>
      </div>
    </div>
  );
}
