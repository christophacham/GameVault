'use client';

import { Game, getReviewColor, getCoverUrl } from '@/lib/api';
import { GameMenu } from './GameMenu';

interface GameCardProps {
  game: Game;
  onClick?: () => void;
  onEdit?: () => void;
  onAdjustMatch?: () => void;
}

export function GameCard({ game, onClick, onEdit, onAdjustMatch }: GameCardProps) {
  const defaultCover = 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="460" height="215" viewBox="0 0 460 215"><rect fill="%231a1a2e" width="460" height="215"/><text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" fill="%234a4a6a" font-size="24">No Cover</text></svg>';

  return (
    <div
      onClick={onClick}
      className="group relative bg-gv-card rounded-lg overflow-hidden cursor-pointer transition-all duration-200 hover:scale-105 hover:shadow-xl hover:shadow-gv-accent/20"
    >
      {/* Cover Image */}
      <div className="aspect-[460/215] w-full overflow-hidden">
        <img
          src={getCoverUrl(game) || defaultCover}
          alt={game.title}
          className="w-full h-full object-cover"
          loading="lazy"
        />
      </div>

      {/* Menu Button (top-right, visible on hover) */}
      {onEdit && (
        <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity z-10">
          <GameMenu onEdit={onEdit} onAdjustMatch={onAdjustMatch} />
        </div>
      )}

      {/* Review Badge (moves left when menu is present) */}
      {game.review_score !== null && (
        <div className={`absolute top-2 ${onEdit ? 'right-12' : 'right-2'} px-2 py-1 rounded text-sm font-bold bg-black/70 ${getReviewColor(game.review_score)}`}>
          {game.review_score}%
        </div>
      )}

      {/* Match Status Badge */}
      {game.match_status === 'pending' && (
        <div className="absolute top-2 left-2 px-2 py-1 rounded text-xs font-medium bg-yellow-500/80 text-black">
          Pending
        </div>
      )}

      {/* Game Info */}
      <div className="p-3">
        <h3 className="font-semibold text-white truncate group-hover:text-gv-accent transition-colors">
          {game.title}
        </h3>

        {/* Genres */}
        {game.genres && game.genres.length > 0 && (
          <div className="mt-1 flex flex-wrap gap-1">
            {game.genres.slice(0, 3).map((genre, i) => (
              <span
                key={i}
                className="text-xs px-2 py-0.5 bg-gv-hover rounded text-gray-400"
              >
                {genre}
              </span>
            ))}
          </div>
        )}

        {/* Review Summary */}
        {game.review_summary && (
          <p className={`mt-1 text-xs ${getReviewColor(game.review_score)}`}>
            {game.review_summary}
          </p>
        )}
      </div>
    </div>
  );
}
