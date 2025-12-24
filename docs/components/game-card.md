---
sidebar_position: 2
---

# GameCard

Displays a single game in the library grid with cover art, title, and review score.

## Import

```tsx
import { GameCard } from '@/components/GameCard';
```

## Props

| Prop | Type | Required | Description |
|------|------|----------|-------------|
| `game` | `GameSummary` | Yes | Game data to display |
| `onUpdate` | `() => void` | Yes | Callback when game is updated |

## GameSummary Type

```typescript
interface GameSummary {
  id: number;
  title: string;
  cover_url?: string;
  local_cover_path?: string;
  genres?: string[];
  review_score?: number;
  review_summary?: string;
  match_status: 'pending' | 'matched' | 'failed' | 'manual';
  user_status?: string;
  hltb_main_mins?: number;
}
```

## Usage

```tsx
<GameCard
  game={{
    id: 1,
    title: "The Witcher 3",
    cover_url: "https://...",
    review_score: 97,
    match_status: "matched"
  }}
  onUpdate={() => refetchGames()}
/>
```

## Features

### Cover Image

Displays game cover with fallback:

```tsx
const coverUrl = game.local_cover_path
  ? `/api/games/${game.id}/cover`
  : game.cover_url
  || '/placeholder-cover.png';
```

### Review Score Badge

Shows Steam review percentage with color coding:

| Score | Color | Label |
|-------|-------|-------|
| 95-100 | Green | Overwhelmingly Positive |
| 80-94 | Light Green | Very Positive |
| 70-79 | Yellow | Mostly Positive |
| 40-69 | Orange | Mixed |
| 0-39 | Red | Negative |

### Match Status Indicator

Visual indicator for games pending enrichment:

```tsx
{game.match_status === 'pending' && (
  <div className="absolute top-2 right-2 bg-yellow-500 rounded-full p-1">
    <ClockIcon className="w-4 h-4" />
  </div>
)}
```

### Context Menu

Opens GameMenu on click:

```tsx
<button
  onClick={() => setMenuOpen(true)}
  className="absolute top-2 right-2 opacity-0 group-hover:opacity-100"
>
  <EllipsisVerticalIcon />
</button>
```

## Styling

### Card Container

```tsx
<div className="
  relative
  group
  bg-gray-800
  rounded-lg
  overflow-hidden
  shadow-lg
  hover:shadow-xl
  hover:scale-105
  transition-all
  duration-200
  cursor-pointer
">
```

### Title Overlay

```tsx
<div className="
  absolute
  bottom-0
  left-0
  right-0
  bg-gradient-to-t
  from-black
  to-transparent
  p-4
">
  <h3 className="text-white font-semibold truncate">
    {game.title}
  </h3>
</div>
```

## Child Components

GameCard renders these child components:

- **GameMenu**: Context menu for actions
- **EditModal**: Edit game metadata
- **AdjustMatchModal**: Fix Steam match

## State Management

```tsx
const [isMenuOpen, setIsMenuOpen] = useState(false);
const [isEditOpen, setIsEditOpen] = useState(false);
const [isAdjustOpen, setIsAdjustOpen] = useState(false);

// Close menu when opening modals
const handleEdit = () => {
  setIsMenuOpen(false);
  setIsEditOpen(true);
};
```

## Event Handling

### Click to Open Menu

```tsx
<div
  onClick={() => setIsMenuOpen(true)}
  onKeyDown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      setIsMenuOpen(true);
    }
  }}
  tabIndex={0}
  role="button"
  aria-haspopup="menu"
>
```

### Update Propagation

When a child modal saves changes:

```tsx
<EditModal
  game={game}
  isOpen={isEditOpen}
  onClose={() => setIsEditOpen(false)}
  onSave={() => {
    setIsEditOpen(false);
    onUpdate();  // Propagate to parent
  }}
/>
```

## Accessibility

- Focusable with keyboard
- Enter/Space opens menu
- Proper ARIA attributes
- Alt text for images

## Example Grid

```tsx
<div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
  {games.map(game => (
    <GameCard
      key={game.id}
      game={game}
      onUpdate={refetch}
    />
  ))}
</div>
```
