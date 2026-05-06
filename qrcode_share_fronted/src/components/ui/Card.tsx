import { type ReactNode } from 'react';

type CardVariant = 'default' | 'cream' | 'feature-pink' | 'feature-teal' | 'feature-lavender' | 'feature-peach' | 'feature-ochre';

interface CardProps {
  children: ReactNode;
  className?: string;
  onClick?: () => void;
  hoverable?: boolean;
  variant?: CardVariant;
}

const variantClasses: Record<CardVariant, string> = {
  default: 'bg-canvas border border-hairline rounded-lg p-4',
  cream: 'bg-surface-card rounded-lg p-6',
  'feature-pink': 'bg-brand-pink text-on-primary rounded-xl p-8',
  'feature-teal': 'bg-brand-teal text-on-dark rounded-xl p-8',
  'feature-lavender': 'bg-brand-lavender text-ink rounded-xl p-8',
  'feature-peach': 'bg-brand-peach text-ink rounded-xl p-8',
  'feature-ochre': 'bg-brand-ochre text-ink rounded-xl p-8',
};

export function Card({ children, className = '', onClick, hoverable = false, variant = 'default' }: CardProps) {
  return (
    <div
      className={`
        ${variantClasses[variant]}
        ${hoverable ? 'cursor-pointer transition-shadow hover:shadow-md' : ''}
        ${onClick ? 'cursor-pointer' : ''}
        ${className}
      `}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => { if (e.key === 'Enter' || e.key === ' ') onClick(); } : undefined}
    >
      {children}
    </div>
  );
}

interface CardHeaderProps {
  title: string;
  subtitle?: string;
  action?: ReactNode;
}

export function CardHeader({ title, subtitle, action }: CardHeaderProps) {
  return (
    <div className="mb-3 flex items-start justify-between">
      <div>
        <h3 className="text-lg font-semibold text-ink">
          {title}
        </h3>
        {subtitle && (
          <p className="text-sm text-muted">{subtitle}</p>
        )}
      </div>
      {action && <div>{action}</div>}
    </div>
  );
}
