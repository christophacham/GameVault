---
sidebar_position: 3
---

# EditModal

Modal dialog for editing game metadata with form validation.

## Import

```tsx
import { EditModal } from '@/components/EditModal';
```

## Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `Game` | Yes | Game to edit |
| `isOpen` | `boolean` | Yes | Modal visibility |
| `onClose` | `() => void` | Yes | Close callback |
| `onSave` | `(game: Game) => void` | Yes | Save success callback |

## Editable Fields

| Field | Type | Validation |
|-------|------|------------|
| Title | Text | Required, non-empty |
| Summary | Textarea | Optional |
| Genres | Text | Comma-separated |
| Developers | Text | Comma-separated |
| Publishers | Text | Comma-separated |
| Release Date | Text | YYYY-MM-DD format |
| Review Score | Number | 0-100 |

## Usage

```tsx
<EditModal
  game={selectedGame}
  isOpen={isEditOpen}
  onClose={() => setIsEditOpen(false)}
  onSave={(updated) => {
    setGames(prev => prev.map(g =>
      g.id === updated.id ? updated : g
    ));
    setIsEditOpen(false);
  }}
/>
```

## Form Validation

### Title Validation

Title is required and cannot be empty:

```tsx
const [title, setTitle] = useState(game.title);

const isFormValid = title.trim().length > 0 && !dateError;

<input
  value={title}
  onChange={e => setTitle(e.target.value)}
  className={title.trim() === '' ? 'border-red-500' : ''}
  required
  aria-required="true"
/>
```

### Date Validation

Release date must be in YYYY-MM-DD format:

```tsx
const [dateError, setDateError] = useState<string | null>(null);

const validateDate = (date: string): boolean => {
  if (!date) return true;  // Optional field
  const regex = /^\d{4}-\d{2}-\d{2}$/;
  if (!regex.test(date)) return false;
  const parsed = new Date(date);
  return !isNaN(parsed.getTime());
};

const handleDateChange = (value: string) => {
  setReleaseDate(value);
  if (value && !validateDate(value)) {
    setDateError('Date must be in YYYY-MM-DD format');
  } else {
    setDateError(null);
  }
};
```

### Score Validation

Review score must be 0-100:

```tsx
const handleScoreChange = (value: string) => {
  const num = parseInt(value, 10);
  if (isNaN(num)) {
    setReviewScore(undefined);
  } else {
    setReviewScore(Math.max(0, Math.min(100, num)));
  }
};
```

## Keyboard Support

### Escape to Close

```tsx
useEffect(() => {
  const handleEscape = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && !saving) {
      onClose();
    }
  };

  if (isOpen) {
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }
}, [isOpen, saving, onClose]);
```

### Tab Navigation

Form fields are naturally tab-navigable in order.

## API Integration

### Save Request

```tsx
const handleSubmit = async (e: FormEvent) => {
  e.preventDefault();

  if (!isFormValid) return;

  setSaving(true);
  try {
    const response = await api.updateGame(game.id, {
      title: title.trim(),
      summary: summary || undefined,
      genres: parseCommaSeparated(genres),
      developers: parseCommaSeparated(developers),
      publishers: parseCommaSeparated(publishers),
      release_date: releaseDate || undefined,
      review_score: reviewScore,
    });

    if (response.success) {
      onSave(response.data);
    } else {
      setError(response.error);
    }
  } finally {
    setSaving(false);
  }
};
```

### Parse Comma-Separated Values

```tsx
const parseCommaSeparated = (value: string): string[] | undefined => {
  if (!value.trim()) return undefined;
  return value
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0);
};
```

## Dual-Write Behavior

When saved, the backend writes to both:
1. **SQLite database** (primary)
2. **`.gamevault/metadata.json`** (backup)

This is handled server-side, transparent to the frontend.

## Styling

### Form Layout

```tsx
<form onSubmit={handleSubmit} className="space-y-4">
  <div>
    <label className="block text-sm font-medium text-gray-300 mb-1">
      Title <span className="text-red-500">*</span>
    </label>
    <input
      type="text"
      value={title}
      onChange={e => setTitle(e.target.value)}
      className="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2"
    />
  </div>
  {/* ... more fields */}
</form>
```

### Error Display

```tsx
{dateError && (
  <p className="text-red-500 text-sm mt-1">{dateError}</p>
)}

{error && (
  <div className="bg-red-900/50 text-red-200 p-3 rounded">
    {error}
  </div>
)}
```

### Save Button States

```tsx
<button
  type="submit"
  disabled={!isFormValid || saving}
  className={`
    px-4 py-2 rounded font-medium
    ${isFormValid && !saving
      ? 'bg-blue-600 hover:bg-blue-700 text-white'
      : 'bg-gray-600 text-gray-400 cursor-not-allowed'
    }
  `}
>
  {saving ? 'Saving...' : 'Save Changes'}
</button>
```

## Testing

```tsx
describe('EditModal', () => {
  it('disables save when title is empty', () => {
    render(<EditModal game={mockGame} isOpen={true} />);

    const input = screen.getByLabelText(/title/i);
    fireEvent.change(input, { target: { value: '' } });

    expect(screen.getByRole('button', { name: /save/i }))
      .toBeDisabled();
  });

  it('shows error for invalid date format', () => {
    render(<EditModal game={mockGame} isOpen={true} />);

    const dateInput = screen.getByLabelText(/release date/i);
    fireEvent.change(dateInput, { target: { value: 'invalid' } });

    expect(screen.getByText(/YYYY-MM-DD/)).toBeInTheDocument();
  });

  it('closes on Escape key', () => {
    const onClose = vi.fn();
    render(<EditModal game={mockGame} isOpen={true} onClose={onClose} />);

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(onClose).toHaveBeenCalled();
  });
});
```
