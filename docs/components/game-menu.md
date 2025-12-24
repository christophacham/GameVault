---
sidebar_position: 5
---

# GameMenu

Context menu with full keyboard navigation and accessibility support.

## Import

```tsx
import { GameMenu } from '@/components/GameMenu';
```

## Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `isOpen` | `boolean` | Yes | Menu visibility |
| `onClose` | `() => void` | Yes | Close callback |
| `onEdit` | `() => void` | Yes | Edit action callback |
| `onAdjustMatch` | `() => void` | No | Adjust match callback |

## Menu Items

| Item | Action | Always Shown |
|------|--------|--------------|
| Edit Details | Opens EditModal | Yes |
| Adjust Match | Opens AdjustMatchModal | Only if `onAdjustMatch` provided |

## Usage

```tsx
const [isMenuOpen, setIsMenuOpen] = useState(false);

<GameMenu
  isOpen={isMenuOpen}
  onClose={() => setIsMenuOpen(false)}
  onEdit={() => {
    setIsMenuOpen(false);
    setIsEditOpen(true);
  }}
  onAdjustMatch={() => {
    setIsMenuOpen(false);
    setIsAdjustOpen(true);
  }}
/>
```

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `Escape` | Close menu |
| `Arrow Down` | Move to next item |
| `Arrow Up` | Move to previous item |
| `Enter` | Activate focused item |
| `Space` | Activate focused item |

### Implementation

```tsx
const [focusedIndex, setFocusedIndex] = useState(0);
const buttonRef = useRef<HTMLButtonElement>(null);

const menuItems = [
  { label: 'Edit Details', action: onEdit },
  ...(onAdjustMatch ? [{ label: 'Adjust Match', action: onAdjustMatch }] : []),
];

const handleKeyDown = useCallback((e: KeyboardEvent) => {
  if (!isOpen) return;

  switch (e.key) {
    case 'Escape':
      e.preventDefault();
      setIsOpen(false);
      buttonRef.current?.focus();
      break;

    case 'ArrowDown':
      e.preventDefault();
      setFocusedIndex(prev => (prev + 1) % menuItems.length);
      break;

    case 'ArrowUp':
      e.preventDefault();
      setFocusedIndex(prev => (prev - 1 + menuItems.length) % menuItems.length);
      break;

    case 'Enter':
    case ' ':
      e.preventDefault();
      menuItems[focusedIndex]?.action();
      setIsOpen(false);
      break;
  }
}, [isOpen, focusedIndex, menuItems]);

useEffect(() => {
  if (isOpen) {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }
}, [isOpen, handleKeyDown]);
```

## Accessibility

### ARIA Attributes

```tsx
<div
  role="menu"
  aria-label="Game actions"
  aria-orientation="vertical"
>
  {menuItems.map((item, index) => (
    <button
      key={item.label}
      role="menuitem"
      tabIndex={focusedIndex === index ? 0 : -1}
      aria-selected={focusedIndex === index}
      onClick={() => {
        item.action();
        onClose();
      }}
    >
      {item.label}
    </button>
  ))}
</div>
```

### Focus Management

When menu opens, first item is focused:

```tsx
useEffect(() => {
  if (isOpen) {
    setFocusedIndex(0);
  }
}, [isOpen]);
```

When menu closes, focus returns to trigger:

```tsx
case 'Escape':
  setIsOpen(false);
  buttonRef.current?.focus();  // Return focus
  break;
```

## Styling

### Menu Container

```tsx
<div className={`
  absolute
  right-0
  top-full
  mt-1
  bg-gray-800
  border
  border-gray-700
  rounded-lg
  shadow-lg
  overflow-hidden
  min-w-[160px]
  z-50
  ${isOpen ? 'block' : 'hidden'}
`}>
```

### Menu Items

```tsx
<button
  className={`
    w-full
    px-4
    py-2
    text-left
    text-sm
    ${focusedIndex === index
      ? 'bg-gray-700 text-white'
      : 'text-gray-300 hover:bg-gray-700'
    }
  `}
>
  {item.label}
</button>
```

## Click Outside to Close

```tsx
const menuRef = useRef<HTMLDivElement>(null);

useEffect(() => {
  const handleClickOutside = (e: MouseEvent) => {
    if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
      onClose();
    }
  };

  if (isOpen) {
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }
}, [isOpen, onClose]);
```

## Testing

```tsx
describe('GameMenu', () => {
  it('renders menu items when open', () => {
    render(
      <GameMenu
        isOpen={true}
        onClose={vi.fn()}
        onEdit={vi.fn()}
        onAdjustMatch={vi.fn()}
      />
    );

    expect(screen.getByRole('menuitem', { name: /edit/i })).toBeInTheDocument();
    expect(screen.getByRole('menuitem', { name: /adjust/i })).toBeInTheDocument();
  });

  it('navigates with arrow keys', () => {
    const onEdit = vi.fn();
    const onAdjust = vi.fn();

    render(
      <GameMenu
        isOpen={true}
        onClose={vi.fn()}
        onEdit={onEdit}
        onAdjustMatch={onAdjust}
      />
    );

    // Focus moves down
    fireEvent.keyDown(document, { key: 'ArrowDown' });

    // Press Enter on second item
    fireEvent.keyDown(document, { key: 'Enter' });

    expect(onAdjust).toHaveBeenCalled();
  });

  it('closes on Escape', () => {
    const onClose = vi.fn();
    render(<GameMenu isOpen={true} onClose={onClose} onEdit={vi.fn()} />);

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(onClose).toHaveBeenCalled();
  });

  it('hides Adjust Match when not provided', () => {
    render(<GameMenu isOpen={true} onClose={vi.fn()} onEdit={vi.fn()} />);

    expect(screen.queryByText(/adjust/i)).not.toBeInTheDocument();
  });
});
```
