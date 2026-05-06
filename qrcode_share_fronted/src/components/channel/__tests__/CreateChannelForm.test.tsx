import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CreateChannelForm } from '../CreateChannelForm';
import type { Channel } from '@/types';

const mockChannel: Channel = {
  id: 'ch_new',
  name: 'New Channel',
  has_password: false,
  link_limitation: undefined,
  channel_type: undefined,
  location: undefined,
  teacher: undefined,
  created_at: '2024-01-01T00:00:00Z',
  subscriber_count: 0,
  message_count: 0,
};

describe('CreateChannelForm', () => {
  it('renders form fields', () => {
    render(
      <CreateChannelForm onSubmit={vi.fn()} onSuccess={vi.fn()} />
    );
    expect(screen.getByText('Channel Name')).toBeDefined();
    expect(screen.getByText('Create Channel')).toBeDefined();
  });

  it('shows only required fields by default', () => {
    render(
      <CreateChannelForm onSubmit={vi.fn()} onSuccess={vi.fn()} />
    );
    expect(screen.getByPlaceholderText('Enter channel name')).toBeDefined();
    expect(screen.getByPlaceholderText('Leave empty for public channel')).toBeDefined();
    expect(screen.queryByPlaceholderText('e.g., classroom, meeting')).toBeNull();
    expect(screen.queryByPlaceholderText('e.g., Room 301')).toBeNull();
  });

  it('shows advanced fields when toggled', () => {
    render(
      <CreateChannelForm onSubmit={vi.fn()} onSuccess={vi.fn()} />
    );
    fireEvent.click(screen.getByText('Advanced options'));
    expect(screen.getByPlaceholderText('e.g., classroom, meeting')).toBeDefined();
    expect(screen.getByPlaceholderText('e.g., Room 301')).toBeDefined();
    expect(screen.getByPlaceholderText('e.g., Prof. Smith')).toBeDefined();
    expect(screen.getByPlaceholderText('e.g., example.com, github.com')).toBeDefined();
  });

  it('hides advanced fields when toggled again', () => {
    render(
      <CreateChannelForm onSubmit={vi.fn()} onSuccess={vi.fn()} />
    );
    fireEvent.click(screen.getByText('Advanced options'));
    expect(screen.getByPlaceholderText('e.g., classroom, meeting')).toBeDefined();
    fireEvent.click(screen.getByText('Advanced options'));
    expect(screen.queryByPlaceholderText('e.g., classroom, meeting')).toBeNull();
  });

  it('shows error when submitting without name', async () => {
    render(
      <CreateChannelForm onSubmit={vi.fn()} onSuccess={vi.fn()} />
    );

    const nameInput = screen.getByPlaceholderText('Enter channel name');
    fireEvent.change(nameInput, { target: { value: '' } });

    const form = nameInput.closest('form')!;
    fireEvent.submit(form);

    await waitFor(() => {
      expect(screen.getByText('Channel name is required')).toBeDefined();
    });
  });

  it('submits correct data when form is valid', async () => {
    const onSubmit = vi.fn().mockResolvedValue(mockChannel);
    const onSuccess = vi.fn();

    render(
      <CreateChannelForm onSubmit={onSubmit} onSuccess={onSuccess} />
    );

    const nameInput = screen.getByPlaceholderText('Enter channel name');
    fireEvent.change(nameInput, { target: { value: 'New Channel' } });
    fireEvent.click(screen.getByText('Create Channel'));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith(
        expect.objectContaining({ name: 'New Channel' })
      );
    });
  });

  it('calls onSuccess after successful creation', async () => {
    const onSubmit = vi.fn().mockResolvedValue(mockChannel);
    const onSuccess = vi.fn();

    render(
      <CreateChannelForm onSubmit={onSubmit} onSuccess={onSuccess} />
    );

    const nameInput = screen.getByPlaceholderText('Enter channel name');
    fireEvent.change(nameInput, { target: { value: 'New Channel' } });
    fireEvent.click(screen.getByText('Create Channel'));

    await waitFor(() => {
      expect(onSuccess).toHaveBeenCalledWith(mockChannel);
    });
  });
});
