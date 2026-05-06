import React from 'react';
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { JoinChannelForm } from '../JoinChannelForm';
import { BrowserRouter } from 'react-router-dom';

function renderWithRouter(ui: React.ReactElement) {
  return render(<BrowserRouter>{ui}</BrowserRouter>);
}

describe('JoinChannelForm', () => {
  it('renders channel ID input', () => {
    renderWithRouter(<JoinChannelForm />);
    expect(screen.getByPlaceholderText('Enter channel ID')).toBeDefined();
  });

  it('shows error for empty channel ID', async () => {
    renderWithRouter(<JoinChannelForm />);

    const input = screen.getByPlaceholderText('Enter channel ID');
    fireEvent.change(input, { target: { value: '' } });

    const form = input.closest('form')!;
    fireEvent.submit(form);

    await waitFor(() => {
      expect(screen.getByText('Channel ID is required')).toBeDefined();
    });
  });

  it('shows error for invalid channel ID characters', async () => {
    renderWithRouter(<JoinChannelForm />);
    const input = screen.getByPlaceholderText('Enter channel ID');
    fireEvent.change(input, { target: { value: 'invalid!@#' } });

    const form = input.closest('form')!;
    fireEvent.submit(form);

    await waitFor(() => {
      expect(
        screen.getByText(/can only contain letters, numbers/)
      ).toBeDefined();
    });
  });

  it('accepts valid channel ID', () => {
    renderWithRouter(<JoinChannelForm />);
    const input = screen.getByPlaceholderText('Enter channel ID');
    fireEvent.change(input, { target: { value: 'test-channel_123' } });
    fireEvent.click(screen.getByText('Join Channel'));
    expect(screen.queryByText(/can only contain/)).toBeNull();
  });
});
