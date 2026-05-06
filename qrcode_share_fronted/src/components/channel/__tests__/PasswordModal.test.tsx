import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { PasswordModal } from '../PasswordModal';

describe('PasswordModal', () => {
  it('renders password input', () => {
    render(<PasswordModal onSubmit={vi.fn()} />);
    expect(screen.getByPlaceholderText('Enter channel password')).toBeDefined();
  });

  it('shows channel name when provided', () => {
    render(<PasswordModal channelName="My Channel" onSubmit={vi.fn()} />);
    expect(screen.getByText(/My Channel/)).toBeDefined();
  });

  it('shows error when submitting empty password', async () => {
    render(<PasswordModal onSubmit={vi.fn()} />);

    const input = screen.getByPlaceholderText('Enter channel password');
    fireEvent.change(input, { target: { value: '' } });

    const form = input.closest('form')!;
    fireEvent.submit(form);

    await waitFor(() => {
      expect(screen.getByText('Password is required')).toBeDefined();
    });
  });

  it('calls onSubmit with password', () => {
    const onSubmit = vi.fn();
    render(<PasswordModal onSubmit={onSubmit} />);

    const input = screen.getByPlaceholderText('Enter channel password');
    fireEvent.change(input, { target: { value: 'secret123' } });
    fireEvent.click(screen.getByText('Join Channel'));

    expect(onSubmit).toHaveBeenCalledWith('secret123');
  });
});
