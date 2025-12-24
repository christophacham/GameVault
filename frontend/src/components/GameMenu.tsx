'use client';

import { useState, useRef, useEffect, useCallback } from 'react';

interface GameMenuProps {
  onEdit: () => void;
  onAdjustMatch?: () => void;
}

export function GameMenu({ onEdit, onAdjustMatch }: GameMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [focusedIndex, setFocusedIndex] = useState(0);
  const menuRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);

  // Get menu items based on props
  const menuItems = [
    { label: 'Edit Details', action: onEdit },
    ...(onAdjustMatch ? [{ label: 'Adjust Match', action: onAdjustMatch }] : []),
  ];

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  // Keyboard navigation
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
        setFocusedIndex((prev) => (prev + 1) % menuItems.length);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setFocusedIndex((prev) => (prev - 1 + menuItems.length) % menuItems.length);
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
      setFocusedIndex(0); // Reset focus when opening
    }
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, handleKeyDown]);

  const handleMenuClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent card click
    setIsOpen(!isOpen);
  };

  const handleEditClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsOpen(false);
    onEdit();
  };

  const handleAdjustMatchClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsOpen(false);
    onAdjustMatch?.();
  };

  return (
    <div ref={menuRef} className="relative">
      {/* Menu Button */}
      <button
        ref={buttonRef}
        onClick={handleMenuClick}
        className="p-1.5 rounded-lg bg-black/60 hover:bg-black/80 text-white transition-colors"
        title="Game options"
        aria-haspopup="true"
        aria-expanded={isOpen}
      >
        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path d="M6 10a2 2 0 11-4 0 2 2 0 014 0zM12 10a2 2 0 11-4 0 2 2 0 014 0zM16 12a2 2 0 100-4 2 2 0 000 4z" />
        </svg>
      </button>

      {/* Dropdown Menu */}
      {isOpen && (
        <div
          className="absolute right-0 top-full mt-1 w-48 bg-gv-card rounded-lg shadow-xl border border-gv-hover z-50 overflow-hidden"
          role="menu"
        >
          <button
            onClick={handleEditClick}
            className={`w-full px-4 py-2.5 text-left text-sm text-white flex items-center gap-2 transition-colors ${focusedIndex === 0 ? 'bg-gv-hover' : 'hover:bg-gv-hover'}`}
            role="menuitem"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
            Edit Details
          </button>
          {onAdjustMatch && (
            <button
              onClick={handleAdjustMatchClick}
              className={`w-full px-4 py-2.5 text-left text-sm text-white flex items-center gap-2 transition-colors ${focusedIndex === 1 ? 'bg-gv-hover' : 'hover:bg-gv-hover'}`}
              role="menuitem"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
              Adjust Match
            </button>
          )}
        </div>
      )}
    </div>
  );
}
