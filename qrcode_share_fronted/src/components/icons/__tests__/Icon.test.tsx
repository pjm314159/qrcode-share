import React from 'react';
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import {
  IconScan,
  IconShare,
  IconOpen,
  IconLock,
  IconCopy,
  IconDownload,
  IconSend,
  IconPaste,
  IconSettings,
  IconBack,
  IconClose,
  IconSearch,
  IconLocation,
  IconUser,
  IconPeople,
  IconWarning,
  IconDanger,
  IconSuccess,
  IconConnecting,
  IconDisconnected,
  IconEmpty,
  IconNotFound,
  IconRocket,
  IconPhone,
  IconPlus,
  IconArrowRight,
  IconChevronLeft,
} from '../index';

const strokeIcons = [
  IconScan,
  IconShare,
  IconOpen,
  IconLock,
  IconCopy,
  IconDownload,
  IconSend,
  IconPaste,
  IconSettings,
  IconBack,
  IconClose,
  IconSearch,
  IconLocation,
  IconUser,
  IconPeople,
  IconWarning,
  IconDanger,
  IconDisconnected,
  IconEmpty,
  IconNotFound,
  IconRocket,
  IconPhone,
  IconPlus,
  IconArrowRight,
  IconChevronLeft,
];

const fillIcons = [IconSuccess];

const customIcons = [IconConnecting];

describe('Icon components', () => {
  describe('stroke-based icons', () => {
    strokeIcons.forEach((IconComponent) => {
      describe(IconComponent.displayName!, () => {
        it('renders with default size 20', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg).not.toBeNull();
          expect(svg?.getAttribute('width')).toBe('20');
          expect(svg?.getAttribute('height')).toBe('20');
        });

        it('renders with custom size', () => {
          const { container } = render(<IconComponent size={32} />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('width')).toBe('32');
          expect(svg?.getAttribute('height')).toBe('32');
        });

        it('applies className', () => {
          const { container } = render(<IconComponent className="text-ink" />);
          const svg = container.querySelector('svg');
          expect(svg?.classList.contains('text-ink')).toBe(true);
        });

        it('has aria-hidden', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('aria-hidden')).toBe('true');
        });

        it('uses stroke currentColor', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('stroke')).toBe('currentColor');
        });
      });
    });
  });

  describe('fill-based icons', () => {
    fillIcons.forEach((IconComponent) => {
      describe(IconComponent.displayName!, () => {
        it('renders with default size 20', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg).not.toBeNull();
          expect(svg?.getAttribute('width')).toBe('20');
          expect(svg?.getAttribute('height')).toBe('20');
        });

        it('renders with custom size', () => {
          const { container } = render(<IconComponent size={48} />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('width')).toBe('48');
          expect(svg?.getAttribute('height')).toBe('48');
        });

        it('applies className', () => {
          const { container } = render(<IconComponent className="text-success" />);
          const svg = container.querySelector('svg');
          expect(svg?.classList.contains('text-success')).toBe(true);
        });

        it('has aria-hidden', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('aria-hidden')).toBe('true');
        });

        it('uses fill currentColor', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('fill')).toBe('currentColor');
        });
      });
    });
  });

  describe('custom icons', () => {
    customIcons.forEach((IconComponent) => {
      describe(IconComponent.displayName!, () => {
        it('renders with default size 20', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg).not.toBeNull();
          expect(svg?.getAttribute('width')).toBe('20');
          expect(svg?.getAttribute('height')).toBe('20');
        });

        it('renders with custom size', () => {
          const { container } = render(<IconComponent size={24} />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('width')).toBe('24');
          expect(svg?.getAttribute('height')).toBe('24');
        });

        it('applies className', () => {
          const { container } = render(<IconComponent className="text-warning" />);
          const svg = container.querySelector('svg');
          expect(svg?.classList.contains('text-warning')).toBe(true);
        });

        it('has aria-hidden', () => {
          const { container } = render(<IconComponent />);
          const svg = container.querySelector('svg');
          expect(svg?.getAttribute('aria-hidden')).toBe('true');
        });
      });
    });
  });
});
