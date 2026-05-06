import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Toggle } from '../Toggle';

describe('Toggle', () => {
  it('renders with label', () => {
    render(<Toggle checked={false} onChange={() => {}} label="Auto-open" />);
    expect(screen.getByText('Auto-open')).toBeDefined();
  });

  it('renders with description', () => {
    render(<Toggle checked={false} onChange={() => {}} label="Auto-open" description="Open links automatically" />);
    expect(screen.getByText('Open links automatically')).toBeDefined();
  });

  it('calls onChange when clicked', () => {
    const onChange = vi.fn();
    render(<Toggle checked={false} onChange={onChange} label="Auto-open" />);
    fireEvent.click(screen.getByRole('switch'));
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it('calls onChange with false when unchecked', () => {
    const onChange = vi.fn();
    render(<Toggle checked={true} onChange={onChange} label="Auto-open" />);
    fireEvent.click(screen.getByRole('switch'));
    expect(onChange).toHaveBeenCalledWith(false);
  });

  it('has correct aria-checked when checked', () => {
    render(<Toggle checked={true} onChange={() => {}} label="Auto-open" />);
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'true');
  });

  it('has correct aria-checked when unchecked', () => {
    render(<Toggle checked={false} onChange={() => {}} label="Auto-open" />);
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'false');
  });

  it('uses default variant track color when checked', () => {
    render(<Toggle checked={true} onChange={() => {}} label="Auto-open" />);
    expect(screen.getByRole('switch')).toHaveClass('bg-ink');
  });

  it('uses danger variant track color when checked', () => {
    render(<Toggle checked={true} onChange={() => {}} label="Auto-open" variant="danger" />);
    expect(screen.getByRole('switch')).toHaveClass('bg-error');
  });

  it('uses hairline color when unchecked', () => {
    render(<Toggle checked={false} onChange={() => {}} label="Auto-open" />);
    expect(screen.getByRole('switch')).toHaveClass('bg-hairline');
  });

  it('does not call onChange when disabled', () => {
    const onChange = vi.fn();
    render(<Toggle checked={false} onChange={onChange} label="Auto-open" disabled />);
    fireEvent.click(screen.getByRole('switch'));
    expect(onChange).not.toHaveBeenCalled();
  });
});
