import { type ReactNode } from 'react';
import { IMAGES } from '@/constants/images';

interface EmptyStateProps {
  title?: string;
  description?: string;
  icon?: ReactNode;
  image?: string;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export function EmptyState({
  title = 'Nothing here yet',
  description,
  icon,
  image,
  action,
}: EmptyStateProps) {
  return (
    <div className="py-12 text-center text-muted">
      {image ? (
        <img
          src={image}
          alt=""
          width={320}
          height={240}
          className="mx-auto rounded-xl opacity-60"
        />
      ) : icon ? (
        icon
      ) : (
        <img
          src={IMAGES.emptyInbox}
          alt=""
          width={320}
          height={240}
          className="mx-auto rounded-xl opacity-60"
        />
      )}
      <p className="mt-4 text-lg font-medium text-ink">{title}</p>
      {description && <p className="mt-1 text-sm">{description}</p>}
      {action && (
        <button
          onClick={action.onClick}
          className="mt-4 rounded-md bg-ink px-5 py-3 text-sm font-semibold text-on-primary hover:bg-ink-active"
        >
          {action.label}
        </button>
      )}
    </div>
  );
}
