import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { SendMessageForm } from '../SendMessageForm';

const mockOnSend = vi.fn().mockResolvedValue(undefined);

describe('SendMessageForm', () => {
  it('renders form fields', () => {
    render(<SendMessageForm onSend={mockOnSend} channelId="ch_001" />);
    expect(screen.getByText('Link URL')).toBeDefined();
    expect(screen.getByText('Send Link')).toBeDefined();
  });

  it('renders link name field', () => {
    render(<SendMessageForm onSend={mockOnSend} channelId="ch_001" />);
    expect(screen.getByText('Link Name (optional)')).toBeDefined();
  });

  it('calls onSend with link and name', async () => {
    mockOnSend.mockClear();
    render(<SendMessageForm onSend={mockOnSend} channelId="ch_001" />);

    const linkInput = screen.getByPlaceholderText('https://example.com');
    fireEvent.change(linkInput, { target: { value: 'https://example.com/test' } });

    const nameInput = screen.getByPlaceholderText('Give this link a name');
    fireEvent.change(nameInput, { target: { value: 'Test Link' } });

    const form = linkInput.closest('form')!;
    fireEvent.submit(form);

    await waitFor(() => {
      expect(mockOnSend).toHaveBeenCalledWith('https://example.com/test', 'Test Link');
    });
  });

  it('clears form after successful send', async () => {
    mockOnSend.mockClear();
    render(<SendMessageForm onSend={mockOnSend} channelId="ch_001" />);

    const linkInput = screen.getByPlaceholderText('https://example.com') as HTMLInputElement;
    fireEvent.change(linkInput, { target: { value: 'https://example.com/test' } });

    const form = linkInput.closest('form')!;
    fireEvent.submit(form);

    await waitFor(() => {
      expect(mockOnSend).toHaveBeenCalled();
    });
  });
});
