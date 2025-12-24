import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EditModal } from './EditModal';
import type { GameDetail } from '@/lib/api';

// Mock the API
vi.mock('@/lib/api', () => ({
  updateGame: vi.fn(),
}));

const mockGame: GameDetail = {
  id: 1,
  title: 'Test Game',
  folder_path: '/games/test',
  folder_name: 'test',
  steam_app_id: 12345,
  summary: 'A test game',
  genres: '["Action", "RPG"]',
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

describe('EditModal', () => {
  const mockOnClose = vi.fn();
  const mockOnSave = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders nothing when closed', () => {
    const { container } = render(
      <EditModal isOpen={false} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );
    expect(container.firstChild).toBeNull();
  });

  it('renders the modal when open', () => {
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );
    expect(screen.getByText('Edit Game Details')).toBeInTheDocument();
  });

  it('populates form fields with game data', () => {
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const titleInput = screen.getByDisplayValue('Test Game');
    expect(titleInput).toBeInTheDocument();
  });

  it('shows required indicator for title', () => {
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    expect(screen.getByText('*')).toBeInTheDocument();
  });

  it('disables save button when title is empty', async () => {
    const user = userEvent.setup();
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const titleInput = screen.getByDisplayValue('Test Game');
    await user.clear(titleInput);

    const saveButton = screen.getByText('Save Changes');
    expect(saveButton).toBeDisabled();
  });

  it('shows error message when title is empty', async () => {
    const user = userEvent.setup();
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const titleInput = screen.getByDisplayValue('Test Game');
    await user.clear(titleInput);

    expect(screen.getByText('Title is required')).toBeInTheDocument();
  });

  it('validates date format correctly', async () => {
    const user = userEvent.setup();
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const dateInput = screen.getByDisplayValue('2024-01-15');
    await user.clear(dateInput);
    await user.type(dateInput, 'invalid-date');

    expect(screen.getByText('Use format YYYY-MM-DD')).toBeInTheDocument();
  });

  it('accepts valid date format', async () => {
    const user = userEvent.setup();
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    const dateInput = screen.getByDisplayValue('2024-01-15');
    await user.clear(dateInput);
    await user.type(dateInput, '2025-12-25');

    expect(screen.queryByText('Use format YYYY-MM-DD')).not.toBeInTheDocument();
  });

  it('closes on Escape key', () => {
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(mockOnClose).toHaveBeenCalled();
  });

  it('closes when Cancel button is clicked', async () => {
    const user = userEvent.setup();
    render(
      <EditModal isOpen={true} game={mockGame} onClose={mockOnClose} onSave={mockOnSave} />
    );

    await user.click(screen.getByText('Cancel'));

    expect(mockOnClose).toHaveBeenCalled();
  });
});
