import { type ReactNode } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { IconPlus } from '@/components/icons';
import { IMAGES } from '@/constants/images';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation();

  return (
    <div className="min-h-screen bg-canvas">
      <header className="sticky top-0 z-10 border-b border-hairline bg-canvas/80 backdrop-blur-sm">
        <div className="mx-auto flex h-16 max-w-5xl items-center justify-between px-4">
          <Link to="/" className="flex items-center gap-2 text-lg font-bold text-ink">
            <img
              src={IMAGES.logo}
              alt="QRcode Share"
              width={28}
              height={28}
              className="rounded-md"
            />
            <span>QRcode Share</span>
          </Link>
          <nav className="flex items-center gap-4">
            <Link
              to="/channels"
              className={`text-sm font-medium transition-colors ${
                location.pathname === '/channels'
                  ? 'text-ink'
                  : 'text-muted hover:text-ink'
              }`}
            >
              Channels
            </Link>
            <Link
              to="/create"
              className="inline-flex items-center gap-1 rounded-md bg-ink px-4 py-2 text-sm font-semibold text-on-primary hover:bg-ink-active"
            >
              <IconPlus size={14} />
              Create
            </Link>
          </nav>
        </div>
      </header>

      <main className="mx-auto max-w-5xl px-4 py-8">{children}</main>

      <footer className="relative border-t border-hairline bg-surface-soft py-6 text-center text-sm text-muted overflow-hidden">
        <img
          src={IMAGES.footerMountains}
          alt=""
          role="presentation"
          className="absolute bottom-0 left-0 w-full h-auto opacity-30 pointer-events-none"
        />
        <p className="relative z-10">QRcode Share - Real-time link sharing via QR code</p>
      </footer>
    </div>
  );
}
