---
sidebar_position: 3
---

# Frontend Architecture

The GameVault frontend is a Next.js 15 application using React 19, built as a static export and embedded into the Rust binary.

## Technology Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Next.js | 15 | React framework with app router |
| React | 19 | UI library |
| TypeScript | 5.x | Type safety |
| TailwindCSS | 3.x | Utility-first styling |
| Vitest | 2.x | Unit testing |
| Testing Library | Latest | Component testing |

## Project Structure

```
frontend/
├── src/
│   ├── app/                 # Next.js app router
│   │   ├── page.tsx         # Home page (game grid)
│   │   ├── layout.tsx       # Root layout
│   │   └── globals.css      # Global styles
│   ├── components/          # React components
│   │   ├── GameCard.tsx     # Game display card
│   │   ├── GameMenu.tsx     # Context menu
│   │   ├── EditModal.tsx    # Edit metadata modal
│   │   ├── AdjustMatchModal.tsx  # Fix Steam match
│   │   ├── SettingsModal.tsx     # Configuration
│   │   ├── EnrichModal.tsx  # Enrichment progress
│   │   ├── SearchBar.tsx    # Search input
│   │   └── StatsBar.tsx     # Library statistics
│   ├── lib/
│   │   └── api.ts           # Backend API client
│   └── test/
│       └── setup.ts         # Vitest setup
├── vitest.config.ts
├── next.config.js
├── tailwind.config.js
└── package.json
```

## Component Hierarchy

```
App (layout.tsx)
└── Home (page.tsx)
    ├── SearchBar
    ├── StatsBar
    ├── SettingsModal
    ├── EnrichModal
    └── GameGrid
        └── GameCard (×N)
            ├── GameMenu
            ├── EditModal
            └── AdjustMatchModal
```

## Key Components

### GameCard

Displays a single game with cover art and hover effects.

```tsx
interface GameCardProps {
  game: GameSummary;
  onUpdate: () => void;
}

function GameCard({ game, onUpdate }: GameCardProps) {
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isEditOpen, setIsEditOpen] = useState(false);
  const [isAdjustOpen, setIsAdjustOpen] = useState(false);

  return (
    <div className="relative group">
      <img src={getCoverUrl(game)} alt={game.title} />
      <div className="overlay">
        <h3>{game.title}</h3>
        {game.review_score && <ReviewBadge score={game.review_score} />}
      </div>
      <GameMenu
        isOpen={isMenuOpen}
        onEdit={() => setIsEditOpen(true)}
        onAdjustMatch={() => setIsAdjustOpen(true)}
      />
      <EditModal isOpen={isEditOpen} game={game} onSave={onUpdate} />
      <AdjustMatchModal isOpen={isAdjustOpen} game={game} onConfirm={onUpdate} />
    </div>
  );
}
```

### GameMenu

Context menu with full keyboard navigation:

```tsx
function GameMenu({ isOpen, onEdit, onAdjustMatch, onClose }) {
  const [focusedIndex, setFocusedIndex] = useState(0);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    switch (e.key) {
      case 'Escape':
        onClose();
        break;
      case 'ArrowDown':
        setFocusedIndex(i => (i + 1) % items.length);
        break;
      case 'ArrowUp':
        setFocusedIndex(i => (i - 1 + items.length) % items.length);
        break;
      case 'Enter':
      case ' ':
        items[focusedIndex].action();
        break;
    }
  }, [focusedIndex, items]);

  useEffect(() => {
    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [isOpen, handleKeyDown]);

  return (
    <div role="menu" aria-label="Game actions">
      {items.map((item, i) => (
        <button
          role="menuitem"
          tabIndex={focusedIndex === i ? 0 : -1}
          className={focusedIndex === i ? 'bg-gray-700' : ''}
          onClick={item.action}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
```

### EditModal

Form validation and keyboard shortcuts:

```tsx
function EditModal({ isOpen, game, onSave, onClose }) {
  const [title, setTitle] = useState(game.title);
  const [dateError, setDateError] = useState<string | null>(null);

  // Escape key handler
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !saving) onClose();
    };
    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      return () => document.removeEventListener('keydown', handleEscape);
    }
  }, [isOpen, saving, onClose]);

  // Date validation (YYYY-MM-DD)
  const validateDate = (date: string): boolean => {
    if (!date) return true;
    const regex = /^\d{4}-\d{2}-\d{2}$/;
    return regex.test(date) && !isNaN(new Date(date).getTime());
  };

  const isFormValid = title.trim().length > 0 && !dateError;

  return (
    <Modal isOpen={isOpen} onClose={onClose}>
      <form onSubmit={handleSubmit}>
        <input
          value={title}
          onChange={e => setTitle(e.target.value)}
          required
          aria-required="true"
        />
        {/* ... other fields */}
        <button disabled={!isFormValid || saving}>
          Save Changes
        </button>
      </form>
    </Modal>
  );
}
```

### AdjustMatchModal

Two-step match correction with preview:

```tsx
function AdjustMatchModal({ game, onConfirm, onClose }) {
  const [steamInput, setSteamInput] = useState('');
  const [preview, setPreview] = useState<RematchResult | null>(null);
  const [step, setStep] = useState<'input' | 'preview'>('input');

  const fetchPreview = async () => {
    const result = await api.rematchGame(game.id, steamInput);
    if (result.success) {
      setPreview(result.data);
      setStep('preview');
    }
  };

  const confirmRematch = async () => {
    await api.confirmRematch(game.id, steamInput);
    onConfirm();
    onClose();
  };

  return (
    <Modal>
      {step === 'input' ? (
        <div>
          <input
            placeholder="Steam URL or App ID"
            value={steamInput}
            onChange={e => setSteamInput(e.target.value)}
          />
          <button onClick={fetchPreview}>Preview</button>
        </div>
      ) : (
        <div>
          <img src={preview.cover_url} />
          <h3>{preview.title}</h3>
          <p>{preview.summary}</p>
          <button onClick={confirmRematch}>Confirm Match</button>
          <button onClick={() => setStep('input')}>Back</button>
        </div>
      )}
    </Modal>
  );
}
```

## API Client

Type-safe API wrapper:

```typescript
// lib/api.ts

const API_BASE = '/api';

interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export async function fetchGames(): Promise<ApiResponse<GameSummary[]>> {
  const res = await fetch(`${API_BASE}/games`);
  return res.json();
}

export async function updateGame(
  id: number,
  updates: UpdateGameRequest
): Promise<ApiResponse<Game>> {
  const res = await fetch(`${API_BASE}/games/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(updates),
  });
  return res.json();
}

export async function rematchGame(
  id: number,
  steamInput: string
): Promise<ApiResponse<RematchResult>> {
  const res = await fetch(`${API_BASE}/games/${id}/match`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ steam_input: steamInput }),
  });
  return res.json();
}
```

## Styling Patterns

### TailwindCSS Dark Theme

```css
/* globals.css */
:root {
  --background: #0a0a0a;
  --foreground: #ededed;
}

body {
  background: var(--background);
  color: var(--foreground);
}
```

### Component Styling

```tsx
<div className="
  bg-gray-900
  rounded-lg
  overflow-hidden
  shadow-lg
  hover:shadow-xl
  transition-shadow
  duration-200
">
```

## Testing

### Vitest Configuration

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    globals: true,
  },
});
```

### Component Tests

```typescript
// EditModal.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { EditModal } from './EditModal';

describe('EditModal', () => {
  it('disables save when title is empty', () => {
    render(<EditModal game={mockGame} isOpen={true} />);

    const titleInput = screen.getByLabelText(/title/i);
    fireEvent.change(titleInput, { target: { value: '' } });

    const saveButton = screen.getByRole('button', { name: /save/i });
    expect(saveButton).toBeDisabled();
  });

  it('closes on Escape key', () => {
    const onClose = vi.fn();
    render(<EditModal game={mockGame} isOpen={true} onClose={onClose} />);

    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalled();
  });
});
```

## Static Export

### Next.js Configuration

```javascript
// next.config.js
module.exports = {
  output: 'export',
  images: { unoptimized: true },
  trailingSlash: true,
};
```

### Build Output

```
frontend/out/
├── index.html
├── 404.html
├── _next/
│   └── static/
│       ├── chunks/
│       └── css/
└── favicon.ico
```

This static output is embedded into the Rust binary at compile time.

## Accessibility

### Keyboard Navigation

All interactive elements support keyboard control:

| Component | Keys |
|-----------|------|
| GameMenu | Arrow Up/Down, Enter, Space, Escape |
| EditModal | Escape to close, Tab to navigate |
| AdjustMatchModal | Escape to close |
| SearchBar | Enter to search, Escape to clear |

### ARIA Attributes

```tsx
<button
  role="menuitem"
  aria-haspopup="true"
  aria-expanded={isOpen}
  aria-label="Game options"
>
```

### Focus Management

```tsx
useEffect(() => {
  if (isOpen) {
    firstInputRef.current?.focus();
  }
}, [isOpen]);
```
