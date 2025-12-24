---
sidebar_position: 4
---

# AdjustMatchModal

Two-step modal for correcting incorrect Steam matches with preview before confirmation.

## Import

```tsx
import { AdjustMatchModal } from '@/components/AdjustMatchModal';
```

## Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `Game` | Yes | Game to rematch |
| `isOpen` | `boolean` | Yes | Modal visibility |
| `onClose` | `() => void` | Yes | Close callback |
| `onConfirm` | `(game: Game) => void` | Yes | Confirmation callback |

## Two-Step Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Step 1        │────▶│   Step 2        │────▶│   Complete      │
│   Input Steam   │     │   Preview Match │     │   Updated       │
│   URL/App ID    │     │   Confirm/Back  │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Usage

```tsx
<AdjustMatchModal
  game={game}
  isOpen={isAdjustOpen}
  onClose={() => setIsAdjustOpen(false)}
  onConfirm={(updated) => {
    refreshGames();
    setIsAdjustOpen(false);
  }}
/>
```

## Step 1: Input

### Accepted Input Formats

| Format | Example |
|--------|---------|
| App ID | `292030` |
| Store URL | `https://store.steampowered.com/app/292030/The_Witcher_3_Wild_Hunt/` |
| Short URL | `store.steampowered.com/app/292030` |

### Input Validation

```tsx
const [steamInput, setSteamInput] = useState('');
const [loading, setLoading] = useState(false);
const [error, setError] = useState<string | null>(null);

const isValidInput = steamInput.trim().length > 0;

const handlePreview = async () => {
  if (!isValidInput) return;

  setLoading(true);
  setError(null);

  const response = await api.rematchGame(game.id, steamInput);

  if (response.success) {
    setPreview(response.data);
    setStep('preview');
  } else {
    setError(response.error || 'Failed to fetch game details');
  }

  setLoading(false);
};
```

## Step 2: Preview

### Preview Data

```typescript
interface RematchResult {
  steam_app_id: number;
  title: string;
  summary?: string;
  genres?: string[];
  developers?: string[];
  publishers?: string[];
  release_date?: string;
  cover_url?: string;
  review_score?: number;
  review_summary?: string;
}
```

### Preview Display

```tsx
{step === 'preview' && preview && (
  <div className="space-y-4">
    {/* Cover image */}
    {preview.cover_url && (
      <img
        src={preview.cover_url}
        alt={preview.title}
        className="w-full h-48 object-cover rounded"
      />
    )}

    {/* Title and details */}
    <div>
      <h3 className="text-xl font-bold">{preview.title}</h3>
      {preview.developers && (
        <p className="text-gray-400">by {preview.developers.join(', ')}</p>
      )}
    </div>

    {/* Summary */}
    {preview.summary && (
      <p className="text-gray-300 text-sm line-clamp-3">
        {preview.summary}
      </p>
    )}

    {/* Metadata */}
    <div className="flex gap-4 text-sm text-gray-400">
      {preview.release_date && <span>{preview.release_date}</span>}
      {preview.review_score && (
        <span className="text-green-400">{preview.review_score}% positive</span>
      )}
    </div>
  </div>
)}
```

## Confirmation

### Confirm Action

```tsx
const handleConfirm = async () => {
  setLoading(true);

  const response = await api.confirmRematch(game.id, steamInput);

  if (response.success) {
    onConfirm(response.data);
  } else {
    setError(response.error || 'Failed to update match');
    setStep('input');  // Go back on error
  }

  setLoading(false);
};
```

### What Happens on Confirm

1. Backend fetches full Steam details
2. Database is updated with new Steam data
3. Cover/background images are cached locally
4. `metadata.json` is written to game folder
5. Updated game is returned to frontend

## Keyboard Support

### Escape to Close

```tsx
useEffect(() => {
  const handleEscape = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && !loading) {
      onClose();
    }
  };

  if (isOpen) {
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }
}, [isOpen, loading, onClose]);
```

## Error Handling

### Common Errors

| Error | Cause | User Action |
|-------|-------|-------------|
| "Invalid Steam URL or App ID" | Malformed input | Check URL/ID format |
| "Could not fetch Steam game details" | App ID doesn't exist | Verify correct game |
| "Failed to update match" | Server error | Try again later |

### Error Display

```tsx
{error && (
  <div className="bg-red-900/50 border border-red-700 text-red-200 p-3 rounded">
    {error}
  </div>
)}
```

## Button States

### Step 1 Buttons

```tsx
<div className="flex gap-2">
  <button
    onClick={onClose}
    className="px-4 py-2 bg-gray-700 rounded"
  >
    Cancel
  </button>
  <button
    onClick={handlePreview}
    disabled={!isValidInput || loading}
    className="px-4 py-2 bg-blue-600 rounded disabled:opacity-50"
  >
    {loading ? 'Loading...' : 'Preview'}
  </button>
</div>
```

### Step 2 Buttons

```tsx
<div className="flex gap-2">
  <button
    onClick={() => setStep('input')}
    disabled={loading}
    className="px-4 py-2 bg-gray-700 rounded"
  >
    Back
  </button>
  <button
    onClick={handleConfirm}
    disabled={loading}
    className="px-4 py-2 bg-green-600 rounded disabled:opacity-50"
  >
    {loading ? 'Updating...' : 'Confirm Match'}
  </button>
</div>
```

## Testing

```tsx
describe('AdjustMatchModal', () => {
  it('shows preview on valid input', async () => {
    vi.mocked(api.rematchGame).mockResolvedValue({
      success: true,
      data: mockPreview
    });

    render(<AdjustMatchModal game={mockGame} isOpen={true} />);

    fireEvent.change(screen.getByPlaceholderText(/steam/i), {
      target: { value: '292030' }
    });
    fireEvent.click(screen.getByRole('button', { name: /preview/i }));

    await waitFor(() => {
      expect(screen.getByText(mockPreview.title)).toBeInTheDocument();
    });
  });

  it('shows error for invalid Steam ID', async () => {
    vi.mocked(api.rematchGame).mockResolvedValue({
      success: false,
      error: 'Invalid Steam URL or App ID'
    });

    render(<AdjustMatchModal game={mockGame} isOpen={true} />);

    fireEvent.change(screen.getByPlaceholderText(/steam/i), {
      target: { value: 'invalid' }
    });
    fireEvent.click(screen.getByRole('button', { name: /preview/i }));

    await waitFor(() => {
      expect(screen.getByText(/invalid/i)).toBeInTheDocument();
    });
  });

  it('closes on Escape', () => {
    const onClose = vi.fn();
    render(<AdjustMatchModal game={mockGame} isOpen={true} onClose={onClose} />);

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(onClose).toHaveBeenCalled();
  });
});
```
