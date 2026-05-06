import React from 'react';
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { DecorativeBlob } from '../DecorativeBlob';

describe('DecorativeBlob', () => {
  it('renders with aria-hidden', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" />
      </div>
    );
    const blob = container.querySelector('[aria-hidden="true"]');
    expect(blob).toBeDefined();
  });

  it('has pointer-events-none class', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" />
      </div>
    );
    const blob = container.querySelector('.pointer-events-none');
    expect(blob).toBeDefined();
  });

  it('applies blur filter', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" />
      </div>
    );
    const blob = container.querySelector('[aria-hidden="true"]') as HTMLElement;
    expect(blob.style.filter).toContain('blur');
  });

  it('applies custom size', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={200} x="10%" y="5%" />
      </div>
    );
    const blob = container.querySelector('[aria-hidden="true"]') as HTMLElement;
    expect(blob.style.width).toBe('200px');
    expect(blob.style.height).toBe('200px');
  });

  it('applies position', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" />
      </div>
    );
    const blob = container.querySelector('[aria-hidden="true"]') as HTMLElement;
    expect(blob.style.left).toBe('10%');
    expect(blob.style.top).toBe('5%');
  });

  it('applies default opacity of 0.15', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" />
      </div>
    );
    const blob = container.querySelector('[aria-hidden="true"]') as HTMLElement;
    expect(blob.style.opacity).toBe('0.15');
  });

  it('applies custom opacity', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" opacity={0.08} />
      </div>
    );
    const blob = container.querySelector('[aria-hidden="true"]') as HTMLElement;
    expect(blob.style.opacity).toBe('0.08');
  });

  it('applies background color', () => {
    const { container } = render(
      <div className="relative">
        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" />
      </div>
    );
    const blob = container.querySelector('[aria-hidden="true"]') as HTMLElement;
    expect(blob.style.backgroundColor).toBe('rgb(184, 164, 237)');
  });
});
