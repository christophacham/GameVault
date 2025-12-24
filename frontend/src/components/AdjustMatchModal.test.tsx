import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AdjustMatchModal } from './AdjustMatchModal';
import * as api from '@/lib/api';
import type { GameDetail } from '@/lib/api';

// Mock the API
vi.mock('@/lib/api', async () => {
  const actual = await vi.importActual('@/lib/api');
  return {
    ...actual,
    previewRematch: vi.fn(),
    confirmRematch: vi.fn(),
  };
});

const mockGame: GameDetail = {
  id: 1,
  title: 'Test Game',
  folder_path: '/games/test',
  folder_name: 'test',
  steam_app_id: 12345,
  summary: 'A test game',
  genres: '["Action"]',
  developers: '["Test Dev"]',
  publishers: '["Test Pub"]',
  release_date: '2024-01-15',
  review_score: 85,
  review_summary: 'Very Positive',
  cover_image: null,
  header_image: null,
  hltb_main_mins: null,
  hltb_extra_mins: null,
  hltb_completionist_mins: null,
  save_path_pattern: null,
  created_at: '2024-01-01',
  updated_at: '2024-01-01',
};

const mockPreviewResult = {
  steam_app_id: 292030,
  title: 'The Witcher 3',
  genres: ['RPG', 'Action'],
  developers: ['CD Projekt RED'],
  publishers: ['CD Projekt'],
  release_date: '2015-05-18',
  review_score: 98,
  review_summary: 'Overwhelmingly Positive',
  summary: 'Epic RPG',
  cover_url: 'https://example.com/cover.jpg',
};

describe('AdjustMatchModal', () => {
  const mockOnClose = vi.fn();
  const mockOnSave = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders nothing when closed', () => {
    const { container } = render(
      <AdjustMatchModal isOpen={false} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );
    expect(container.firstChild).toBeNull();
  });

  it('renders the modal when open', () => {
    render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );
    expect(screen.getByText('Adjust Match')).toBeInTheDocument();
    expect(screen.getByText(/Currently matching:/)).toBeInTheDocument();
  });

  it('shows current game title', () => {
    render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );
    expect(screen.getByText('Test Game')).toBeInTheDocument();
  });

  it('toggles help section', async () => {
    const user = userEvent.setup();
    render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    // Help should be hidden initially
    expect(screen.queryByText('How to find the correct game:')).not.toBeInTheDocument();

    // Click help button
    await user.click(screen.getByTitle('Show help'));

    expect(screen.getByText('How to find the correct game:')).toBeInTheDocument();
  });

  it('calls preview API with Steam input', async () => {
    const user = userEvent.setup();
    vi.mocked(api.previewRematch).mockResolvedValue(mockPreviewResult);

    render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const input = screen.getByPlaceholderText(/292030/);
    await user.type(input, '292030');
    await user.click(screen.getByText('Preview'));

    await waitFor(() => {
      expect(api.previewRematch).toHaveBeenCalledWith(1, '292030');
    });
  });

  it('displays preview result', async () => {
    const user = userEvent.setup();
    vi.mocked(api.previewRematch).mockResolvedValue(mockPreviewResult);

    render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const input = screen.getByPlaceholderText(/292030/);
    await user.type(input, '292030');
    await user.click(screen.getByText('Preview'));

    await waitFor(() => {
      expect(screen.getByText('The Witcher 3')).toBeInTheDocument();
      expect(screen.getByText('Confirm Match')).toBeInTheDocument();
    });
  });

  it('shows error on preview failure', async () => {
    const user = userEvent.setup();
    vi.mocked(api.previewRematch).mockRejectedValue(new Error('Game not found'));

    render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const input = screen.getByPlaceholderText(/292030/);
    await user.type(input, 'invalid');
    await user.click(screen.getByText('Preview'));

    await waitFor(() => {
      expect(screen.getByText('Game not found')).toBeInTheDocument();
    });
  });

  it('closes on Escape key', () => {
    render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(mockOnClose).toHaveBeenCalled();
  });

  it('resets state when closed', async () => {
    const user = userEvent.setup();
    vi.mocked(api.previewRematch).mockResolvedValue(mockPreviewResult);

    const { rerender } = render(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    // Type something and preview
    const input = screen.getByPlaceholderText(/292030/);
    await user.type(input, '292030');
    await user.click(screen.getByText('Preview'));

    await waitFor(() => {
      expect(screen.getByText('The Witcher 3')).toBeInTheDocument();
    });

    // Close and reopen
    await user.click(screen.getByText('Cancel'));

    rerender(
      <AdjustMatchModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    // Preview should be gone
    expect(screen.queryByText('The Witcher 3')).not.toBeInTheDocument();
  });
});
