---
sidebar_position: 6
---

# SettingsModal

Configuration modal for managing GameVault settings, including paths and server options.

## Import

```tsx
import { SettingsModal } from '@/components/SettingsModal';
```

## Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `isOpen` | `boolean` | Yes | Modal visibility |
| `onClose` | `() => void` | Yes | Close callback |

## Settings Fields

| Field | Type | Validation | Restart Required |
|-------|------|------------|------------------|
| Game Library | Path | Must exist | No |
| Cache Directory | Path | Auto-created | No |
| Port | Number | 1024-65535 | **Yes** |
| Auto-open Browser | Boolean | - | No |

## Usage

```tsx
const [isSettingsOpen, setIsSettingsOpen] = useState(false);

<button onClick={() => setIsSettingsOpen(true)}>
  <CogIcon />
</button>

<SettingsModal
  isOpen={isSettingsOpen}
  onClose={() => setIsSettingsOpen(false)}
/>
```

## Form State

```tsx
interface ConfigState {
  gameLibrary: string;
  cache: string;
  port: number;
  autoOpenBrowser: boolean;
}

const [config, setConfig] = useState<ConfigState | null>(null);
const [loading, setLoading] = useState(true);
const [saving, setSaving] = useState(false);
const [error, setError] = useState<string | null>(null);
```

## API Integration

### Load Settings

```tsx
useEffect(() => {
  if (isOpen) {
    api.getConfig().then(response => {
      if (response.success) {
        setConfig({
          gameLibrary: response.data.paths.game_library,
          cache: response.data.paths.cache,
          port: response.data.server.port,
          autoOpenBrowser: response.data.server.auto_open_browser,
        });
      }
      setLoading(false);
    });
  }
}, [isOpen]);
```

### Save Settings

```tsx
const handleSave = async () => {
  if (!config) return;

  setSaving(true);
  setError(null);

  const response = await api.updateConfig({
    game_library: config.gameLibrary,
    cache: config.cache,
    port: config.port,
    auto_open_browser: config.autoOpenBrowser,
  });

  if (response.success) {
    if (response.data.restart_required) {
      setShowRestartPrompt(true);
    } else {
      onClose();
    }
  } else {
    setError(response.error);
  }

  setSaving(false);
};
```

## Validation

### Path Validation

Game library path must exist and be a directory:

```tsx
const [pathValid, setPathValid] = useState<boolean | null>(null);

const validatePath = async (path: string) => {
  // Server-side validation via config status endpoint
  const response = await api.getConfigStatus();
  setPathValid(response.data?.game_library_exists ?? false);
};
```

### Port Validation

```tsx
const isPortValid = config.port >= 1024 && config.port <= 65535;

{!isPortValid && (
  <p className="text-red-500 text-sm">Port must be between 1024 and 65535</p>
)}
```

## Restart Flow

When port changes, restart is required:

```tsx
const [showRestartPrompt, setShowRestartPrompt] = useState(false);

const handleRestart = async () => {
  await api.restart();
  // Server will restart, connection will be lost
  // User needs to refresh after a moment
};

{showRestartPrompt && (
  <div className="bg-yellow-900/50 p-4 rounded">
    <p>Port change requires restart. Restart now?</p>
    <div className="flex gap-2 mt-2">
      <button onClick={handleRestart}>Restart</button>
      <button onClick={onClose}>Later</button>
    </div>
  </div>
)}
```

## Shutdown

```tsx
const handleShutdown = async () => {
  if (confirm('Are you sure you want to shut down GameVault?')) {
    await api.shutdown();
    // Application will exit
  }
};
```

## Form Layout

```tsx
<form onSubmit={handleSubmit} className="space-y-6">
  {/* Paths Section */}
  <section>
    <h3 className="text-lg font-medium mb-4">Paths</h3>

    <div className="space-y-4">
      <div>
        <label className="block text-sm text-gray-400 mb-1">
          Game Library
        </label>
        <input
          type="text"
          value={config.gameLibrary}
          onChange={e => setConfig({...config, gameLibrary: e.target.value})}
          className="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2"
          placeholder="D:\Games"
        />
        {pathValid === false && (
          <p className="text-red-500 text-sm mt-1">Path does not exist</p>
        )}
      </div>

      <div>
        <label className="block text-sm text-gray-400 mb-1">
          Cache Directory
        </label>
        <input
          type="text"
          value={config.cache}
          onChange={e => setConfig({...config, cache: e.target.value})}
          className="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2"
          placeholder="./cache"
        />
      </div>
    </div>
  </section>

  {/* Server Section */}
  <section>
    <h3 className="text-lg font-medium mb-4">Server</h3>

    <div className="space-y-4">
      <div>
        <label className="block text-sm text-gray-400 mb-1">
          Port
        </label>
        <input
          type="number"
          value={config.port}
          onChange={e => setConfig({...config, port: parseInt(e.target.value)})}
          min={1024}
          max={65535}
          className="w-32 bg-gray-800 border border-gray-700 rounded px-3 py-2"
        />
      </div>

      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="autoOpen"
          checked={config.autoOpenBrowser}
          onChange={e => setConfig({...config, autoOpenBrowser: e.target.checked})}
        />
        <label htmlFor="autoOpen" className="text-sm">
          Open browser on startup
        </label>
      </div>
    </div>
  </section>

  {/* Actions */}
  <div className="flex justify-between pt-4 border-t border-gray-700">
    <button
      type="button"
      onClick={handleShutdown}
      className="text-red-400 hover:text-red-300"
    >
      Shutdown
    </button>

    <div className="flex gap-2">
      <button type="button" onClick={onClose}>Cancel</button>
      <button
        type="submit"
        disabled={saving || !isFormValid}
        className="bg-blue-600 px-4 py-2 rounded"
      >
        {saving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </div>
</form>
```

## Loading State

```tsx
{loading ? (
  <div className="flex items-center justify-center h-48">
    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white" />
  </div>
) : (
  <form>...</form>
)}
```

## Testing

```tsx
describe('SettingsModal', () => {
  it('loads current config on open', async () => {
    vi.mocked(api.getConfig).mockResolvedValue({
      success: true,
      data: mockConfig
    });

    render(<SettingsModal isOpen={true} onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByDisplayValue(mockConfig.paths.game_library))
        .toBeInTheDocument();
    });
  });

  it('shows validation error for invalid path', async () => {
    vi.mocked(api.updateConfig).mockResolvedValue({
      success: false,
      error: 'Path does not exist'
    });

    render(<SettingsModal isOpen={true} onClose={vi.fn()} />);

    // ... fill form and submit

    await waitFor(() => {
      expect(screen.getByText(/does not exist/i)).toBeInTheDocument();
    });
  });
});
```
