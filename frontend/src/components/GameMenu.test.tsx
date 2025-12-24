import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { GameMenu } from './GameMenu';

describe('GameMenu', () => {
  const mockOnEdit = vi.fn();
  const mockOnAdjustMatch = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders menu button', () => {
    render(<GameMenu onEdit={mockOnEdit} />);
    expect(screen.getByTitle('Game options')).toBeInTheDocument();
  });

  it('opens menu on click', async () => {
    const user = userEvent.setup();
    render(<GameMenu onEdit={mockOnEdit} onAdjustMatch={mockOnAdjustMatch} />);

    await user.click(screen.getByTitle('Game options'));

    expect(screen.getByText('Edit Details')).toBeInTheDocument();
    expect(screen.getByText('Adjust Match')).toBeInTheDocument();
  });

  it('calls onEdit when Edit Details is clicked', async () => {
    const user = userEvent.setup();
    render(<GameMenu onEdit={mockOnEdit} />);

    await user.click(screen.getByTitle('Game options'));
    await user.click(screen.getByText('Edit Details'));

    expect(mockOnEdit).toHaveBeenCalled();
  });

  it('calls onAdjustMatch when Adjust Match is clicked', async () => {
    const user = userEvent.setup();
    render(<GameMenu onEdit={mockOnEdit} onAdjustMatch={mockOnAdjustMatch} />);

    await user.click(screen.getByTitle('Game options'));
    await user.click(screen.getByText('Adjust Match'));

    expect(mockOnAdjustMatch).toHaveBeenCalled();
  });

  it('hides Adjust Match when callback not provided', async () => {
    const user = userEvent.setup();
    render(<GameMenu onEdit={mockOnEdit} />);

    await user.click(screen.getByTitle('Game options'));

    expect(screen.queryByText('Adjust Match')).not.toBeInTheDocument();
  });

  it('closes menu on Escape key', async () => {
    const user = userEvent.setup();
    render(<GameMenu onEdit={mockOnEdit} onAdjustMatch={mockOnAdjustMatch} />);

    await user.click(screen.getByTitle('Game options'));
    expect(screen.getByText('Edit Details')).toBeInTheDocument();

    fireEvent.keyDown(document, { key: 'Escape' });

    expect(screen.queryByText('Edit Details')).not.toBeInTheDocument();
  });

  it('selects first item with Enter key', async () => {
    const user = userEvent.setup();
    render(<GameMenu onEdit={mockOnEdit} onAdjustMatch={mockOnAdjustMatch} />);

    await user.click(screen.getByTitle('Game options'));

    // Enter should select first item (Edit Details) since focusedIndex starts at 0
    fireEvent.keyDown(document, { key: 'Enter' });

    // Should have called onEdit (first item)
    expect(mockOnEdit).toHaveBeenCalled();
  });

  it('closes menu when clicking outside', async () => {
    const user = userEvent.setup();
    render(
      <div>
        <GameMenu onEdit={mockOnEdit} />
        <button data-testid="outside">Outside</button>
      </div>
    );

    await user.click(screen.getByTitle('Game options'));
    expect(screen.getByText('Edit Details')).toBeInTheDocument();

    fireEvent.mouseDown(screen.getByTestId('outside'));

    expect(screen.queryByText('Edit Details')).not.toBeInTheDocument();
  });

  it('has proper aria attributes', async () => {
    const user = userEvent.setup();
    render(<GameMenu onEdit={mockOnEdit} />);

    const button = screen.getByTitle('Game options');
    expect(button).toHaveAttribute('aria-haspopup', 'true');
    expect(button).toHaveAttribute('aria-expanded', 'false');

    await user.click(button);
    expect(button).toHaveAttribute('aria-expanded', 'true');
  });
});
