---
sidebar_position: 1
---

# Components Overview

GameVault's frontend is built with React 19 components following modern patterns for accessibility, keyboard navigation, and type safety.

## Component Library

| Component | Purpose | Props | Key Features |
|-----------|---------|-------|--------------|
| [GameCard](./game-card) | Display game in grid | `game`, `onUpdate` | Cover image, review badge, context menu |
| [GameMenu](./game-menu) | Context menu | `isOpen`, `onEdit`, `onAdjustMatch` | Full keyboard navigation |
| [EditModal](./edit-modal) | Edit metadata | `game`, `isOpen`, `onSave` | Form validation, Escape key |
| [AdjustMatchModal](./adjust-match-modal) | Fix Steam match | `game`, `isOpen`, `onConfirm` | Two-step preview/confirm |
| [SettingsModal](./settings-modal) | Configuration | `isOpen`, `onClose` | Path validation, restart |
| SearchBar | Search input | `value`, `onChange` | Debounced search |
| StatsBar | Library stats | `stats` | Matched/pending counts |
| EnrichModal | Enrichment progress | `isOpen`, `onClose` | Progress tracking |

## Design Patterns

### Controlled Components

All form components are controlled with React state:

```tsx
const [value, setValue] = useState('');

<input
  value={value}
  onChange={e => setValue(e.target.value)}
/>
```

### Modal Pattern

Modals share a consistent structure:

```tsx
interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  children: React.ReactNode;
}

function Modal({ isOpen, onClose, children }: ModalProps) {
  // Escape key handler
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (isOpen) {
      document.addEventListener('keydown', handler);
      return () => document.removeEventListener('keydown', handler);
    }
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center">
      <div className="bg-gray-900 rounded-lg p-6 max-w-lg w-full">
        {children}
      </div>
    </div>
  );
}
```

### Callback Props

Components communicate via callback props:

```tsx
<GameCard
  game={game}
  onUpdate={() => refetchGames()}  // Parent handles refresh
/>
```

### Loading States

Async operations show loading indicators:

```tsx
const [loading, setLoading] = useState(false);

const handleSave = async () => {
  setLoading(true);
  try {
    await api.updateGame(id, data);
    onSuccess();
  } finally {
    setLoading(false);
  }
};

<button disabled={loading}>
  {loading ? 'Saving...' : 'Save'}
</button>
```

## Accessibility Standards

### ARIA Attributes

```tsx
<button
  role="menuitem"
  aria-haspopup="true"
  aria-expanded={isOpen}
  aria-label="Open game menu"
>
```

### Focus Management

```tsx
const inputRef = useRef<HTMLInputElement>(null);

useEffect(() => {
  if (isOpen) {
    inputRef.current?.focus();
  }
}, [isOpen]);
```

### Keyboard Navigation

All interactive elements support keyboard:

| Key | Action |
|-----|--------|
| Tab | Move focus |
| Enter/Space | Activate |
| Escape | Close/Cancel |
| Arrow Keys | Navigate lists |

## Styling Conventions

### TailwindCSS Classes

```tsx
<div className="
  bg-gray-900       // Background
  rounded-lg        // Border radius
  p-4               // Padding
  hover:bg-gray-800 // Hover state
  transition-colors // Animation
  duration-200      // Animation speed
">
```

### Dark Theme

All components use dark theme colors:

- Background: `bg-gray-900`, `bg-gray-800`
- Text: `text-white`, `text-gray-300`
- Borders: `border-gray-700`
- Accents: `bg-blue-600`, `bg-green-600`

### Responsive Design

```tsx
<div className="
  grid
  grid-cols-2      // Mobile: 2 columns
  md:grid-cols-4   // Tablet: 4 columns
  lg:grid-cols-6   // Desktop: 6 columns
  gap-4
">
```

## Testing Approach

### Component Tests

Each component has a test file:

```
components/
├── GameCard.tsx
├── GameCard.test.tsx
├── EditModal.tsx
├── EditModal.test.tsx
└── ...
```

### Test Structure

```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { vi } from 'vitest';

describe('ComponentName', () => {
  const mockProps = { /* ... */ };

  it('renders correctly', () => {
    render(<Component {...mockProps} />);
    expect(screen.getByText('Expected Text')).toBeInTheDocument();
  });

  it('calls callback on action', () => {
    const onAction = vi.fn();
    render(<Component {...mockProps} onAction={onAction} />);
    fireEvent.click(screen.getByRole('button'));
    expect(onAction).toHaveBeenCalled();
  });
});
```

## State Management

### Local State

Components manage their own UI state:

```tsx
const [isOpen, setIsOpen] = useState(false);
const [formData, setFormData] = useState(initialData);
```

### Derived State

Computed values are derived from props:

```tsx
const isFormValid = useMemo(() => {
  return title.trim().length > 0 && !dateError;
}, [title, dateError]);
```

### Effect Cleanup

Effects properly clean up event listeners:

```tsx
useEffect(() => {
  const handler = () => { /* ... */ };
  document.addEventListener('keydown', handler);
  return () => document.removeEventListener('keydown', handler);
}, [dependencies]);
```

## API Integration

### Fetching Data

```tsx
const [games, setGames] = useState<Game[]>([]);
const [loading, setLoading] = useState(true);

useEffect(() => {
  api.fetchGames().then(response => {
    if (response.success) {
      setGames(response.data);
    }
    setLoading(false);
  });
}, []);
```

### Mutating Data

```tsx
const handleUpdate = async (updates: UpdateRequest) => {
  const response = await api.updateGame(game.id, updates);
  if (response.success) {
    onUpdate(); // Trigger parent refresh
    onClose();
  } else {
    setError(response.error);
  }
};
```
