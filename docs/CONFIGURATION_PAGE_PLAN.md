# GameVault Configuration Page Implementation Plan

> **Research conducted with:** Google Gemini 2.5 Pro (thinkdeep)
> **Confidence:** High
> **Date:** December 2024

---

## Overview

Add a Settings/Configuration page to GameVault that allows users to configure:
- Game library path (most important)
- Cache directory
- Server port
- Auto-open browser setting

## Architecture Decision

**Approach:** Modal-based settings with backend API

**Rationale:**
- Follows existing UI pattern (Edit, Enrich, AdjustMatch modals)
- File-based persistence (config.toml) ensures portability
- Backend validation provides security
- Atomic file writes prevent corruption

---

## Implementation Plan

### Phase 1: Backend API

#### 1.1 Update config.rs

Add `Serialize` derive and write functionality:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub paths: PathsConfig,
    pub server: ServerConfig,
}

// Add function to write config atomically
pub fn write_config(config: &AppConfig) -> anyhow::Result<()> {
    let exe_dir = get_exe_directory();
    let config_path = exe_dir.join("config.toml");
    let temp_path = exe_dir.join("config.toml.tmp");

    // Write to temp file first
    let toml_string = toml::to_string_pretty(config)?;
    std::fs::write(&temp_path, toml_string)?;

    // Atomic rename
    std::fs::rename(&temp_path, &config_path)?;

    Ok(())
}
```

#### 1.2 Add Config Handlers (handlers.rs)

```rust
// GET /api/config
pub async fn get_config() -> impl IntoResponse {
    match AppConfig::load() {
        Ok(config) => {
            let response = ConfigResponse {
                paths: PathsResponse {
                    game_library: config.games_path().to_string_lossy().to_string(),
                    cache: config.cache_path().to_string_lossy().to_string(),
                    game_library_exists: config.games_path().is_dir(),
                    cache_exists: config.cache_path().is_dir(),
                },
                server: ServerResponse {
                    port: config.server.port,
                    auto_open_browser: config.server.auto_open_browser,
                },
            };
            ApiResponse::success(response)
        }
        Err(e) => ApiResponse::error(format!("Failed to load config: {}", e)),
    }
}

// PUT /api/config
pub async fn update_config(
    Json(payload): Json<ConfigUpdateRequest>,
) -> impl IntoResponse {
    // Validate paths exist
    let game_path = PathBuf::from(&payload.game_library);
    if !game_path.is_dir() {
        return ApiResponse::error("Game library path does not exist or is not a directory");
    }

    // Load current config to check for port changes
    let current_config = AppConfig::load().ok();
    let restart_required = current_config
        .map(|c| c.server.port != payload.port)
        .unwrap_or(false);

    // Build new config
    let new_config = AppConfig { ... };

    // Atomic write
    match write_config(&new_config) {
        Ok(_) => ApiResponse::success(ConfigUpdateResponse {
            success: true,
            restart_required
        }),
        Err(e) => ApiResponse::error(format!("Failed to save config: {}", e)),
    }
}
```

#### 1.3 Add Routes (main.rs)

```rust
let config_routes = Router::new()
    .route("/config", get(handlers::get_config))
    .route("/config", put(handlers::update_config));

let api_routes = Router::new()
    // ... existing routes ...
    .merge(config_routes);
```

### Phase 2: Frontend

#### 2.1 Update api.ts

```typescript
export interface ConfigPaths {
  game_library: string;
  cache: string;
  game_library_exists: boolean;
  cache_exists: boolean;
}

export interface ConfigServer {
  port: number;
  auto_open_browser: boolean;
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
```

#### 2.2 Create SettingsModal.tsx

```tsx
'use client';

import { useState, useEffect } from 'react';
import { Config, getConfig, updateConfig } from '@/lib/api';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form state
  const [gameLibrary, setGameLibrary] = useState('');
  const [cachePath, setCachePath] = useState('');
  const [port, setPort] = useState(3000);
  const [autoOpenBrowser, setAutoOpenBrowser] = useState(true);

  useEffect(() => {
    if (isOpen) {
      loadConfig();
    }
  }, [isOpen]);

  const loadConfig = async () => {
    try {
      setLoading(true);
      const data = await getConfig();
      setConfig(data);
      setGameLibrary(data.paths.game_library);
      setCachePath(data.paths.cache);
      setPort(data.server.port);
      setAutoOpenBrowser(data.server.auto_open_browser);
    } catch (err) {
      setError('Failed to load configuration');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setSaving(true);
      setError(null);

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
      setError(err instanceof Error ? err.message : 'Failed to save');
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gv-card rounded-lg w-full max-w-lg mx-4">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gv-hover">
          <h2 className="text-xl font-semibold text-white">Settings</h2>
          <button onClick={onClose}>X</button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* Game Library Path */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Game Library Path
            </label>
            <input
              type="text"
              value={gameLibrary}
              onChange={(e) => setGameLibrary(e.target.value)}
              className="w-full px-3 py-2 bg-gv-dark border border-gv-hover rounded"
            />
            {config && !config.paths.game_library_exists && (
              <p className="text-red-400 text-sm mt-1">Path does not exist</p>
            )}
          </div>

          {/* Cache Path */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Cache Directory
            </label>
            <input
              type="text"
              value={cachePath}
              onChange={(e) => setCachePath(e.target.value)}
              className="w-full px-3 py-2 bg-gv-dark border border-gv-hover rounded"
            />
          </div>

          {/* Port */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Server Port
            </label>
            <input
              type="number"
              value={port}
              onChange={(e) => setPort(parseInt(e.target.value))}
              className="w-full px-3 py-2 bg-gv-dark border border-gv-hover rounded"
            />
            <p className="text-yellow-400 text-sm mt-1">
              Changes require restart
            </p>
          </div>

          {/* Auto-open browser */}
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={autoOpenBrowser}
              onChange={(e) => setAutoOpenBrowser(e.target.checked)}
            />
            <label className="text-gray-300">
              Auto-open browser on startup
            </label>
          </div>

          {error && (
            <p className="text-red-400">{error}</p>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-2 p-4 border-t border-gv-hover">
          <button onClick={onClose}>Cancel</button>
          <button onClick={handleSave} disabled={saving}>
            {saving ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  );
}
```

#### 2.3 Update page.tsx

Add settings button to header:

```tsx
// Add state
const [settingsModalOpen, setSettingsModalOpen] = useState(false);

// Add button in header
<button
  onClick={() => setSettingsModalOpen(true)}
  className="p-2 text-gray-400 hover:text-white"
  title="Settings"
>
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
      d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
  </svg>
</button>

// Add modal
<SettingsModal
  isOpen={settingsModalOpen}
  onClose={() => setSettingsModalOpen(false)}
/>
```

---

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `backend/src/config.rs` | Modify | Add Serialize, write_config() |
| `backend/src/handlers.rs` | Modify | Add get_config, update_config |
| `backend/src/main.rs` | Modify | Add config routes |
| `frontend/src/lib/api.ts` | Modify | Add config types and functions |
| `frontend/src/components/SettingsModal.tsx` | Create | New settings modal component |
| `frontend/src/app/page.tsx` | Modify | Add settings button and modal |

---

## Security Considerations

1. **Path Traversal:** Backend validates paths are directories, not files
2. **Atomic Writes:** Prevents config file corruption
3. **Local-only:** Config API only accessible on localhost (default bind)

---

## UI Mockup

```
┌─────────────────────────────────────────┐
│ Settings                            [X] │
├─────────────────────────────────────────┤
│                                         │
│ Game Library Path                       │
│ ┌─────────────────────────────────────┐ │
│ │ D:\Games                            │ │
│ └─────────────────────────────────────┘ │
│ ✓ Path exists                           │
│                                         │
│ Cache Directory                         │
│ ┌─────────────────────────────────────┐ │
│ │ ./cache                             │ │
│ └─────────────────────────────────────┘ │
│                                         │
│ Server Port                             │
│ ┌─────────────────────────────────────┐ │
│ │ 3000                                │ │
│ └─────────────────────────────────────┘ │
│ ⚠ Changes require restart               │
│                                         │
│ [x] Auto-open browser on startup        │
│                                         │
├─────────────────────────────────────────┤
│              [Cancel] [Save]            │
└─────────────────────────────────────────┘
```

---

## Implementation Order

1. Backend: Add Serialize to config structs
2. Backend: Add write_config function
3. Backend: Add get_config handler
4. Backend: Add update_config handler
5. Backend: Add routes
6. Frontend: Add api.ts types and functions
7. Frontend: Create SettingsModal.tsx
8. Frontend: Add settings button to page.tsx
9. Test: Build and verify
10. Commit

---

*Plan generated with AI assistance from Google Gemini 2.5 Pro*
