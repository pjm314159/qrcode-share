import React from 'react';
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { Skeleton } from '../Skeleton';

describe('Skeleton', () => {
  it('renders text variant by default', () => {
    const { container } = render(<Skeleton />);
    expect(container.firstChild).toHaveClass('animate-pulse');
  });

  it('renders card variant', () => {
    const { container } = render(<Skeleton variant="card" />);
    expect(container.firstChild).toHaveClass('h-32');
  });

  it('renders circle variant', () => {
    const { container } = render(<Skeleton variant="circle" />);
    expect(container.firstChild).toHaveClass('rounded-full');
  });

  it('renders rect variant', () => {
    const { container } = render(<Skeleton variant="rect" />);
    expect(container.firstChild).toHaveClass('rounded-md');
  });

  it('applies custom width and height', () => {
    const { container } = render(<Skeleton width={200} height={20} />);
    const el = container.firstChild as HTMLElement;
    expect(el.style.width).toBe('200px');
    expect(el.style.height).toBe('20px');
  });

  it('has aria-hidden', () => {
    const { container } = render(<Skeleton />);
    expect(container.firstChild).toHaveAttribute('aria-hidden', 'true');
  });

  it('applies custom className', () => {
    const { container } = render(<Skeleton className="mt-4" />);
    expect(container.firstChild).toHaveClass('mt-4');
  });
});
