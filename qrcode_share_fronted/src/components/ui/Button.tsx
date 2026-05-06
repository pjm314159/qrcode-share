import { type ReactNode } from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'on-color';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  children: ReactNode;
}

const variantClasses: Record<string, string> = {
  primary:
    'bg-ink text-on-primary hover:bg-ink-active active:bg-ink-active focus:ring-ink/30',
  secondary:
    'bg-canvas text-ink border border-hairline hover:bg-surface-soft active:bg-surface-card focus:ring-ink/30',
  danger:
    'bg-error text-on-primary hover:bg-error/90 active:bg-error/80 focus:ring-error/30',
  ghost:
    'bg-transparent text-muted hover:bg-surface-soft active:bg-surface-card focus:ring-ink/30',
  'on-color':
    'bg-canvas text-ink hover:bg-surface-soft active:bg-surface-card focus:ring-ink/30',
};

const sizeClasses: Record<string, string> = {
  sm: 'px-3 py-1.5 text-sm h-9',
  md: 'px-5 py-3 text-sm h-11',
  lg: 'px-6 py-3 text-base h-12',
};

export function Button({
  variant = 'primary',
  size = 'md',
  loading = false,
  disabled,
  children,
  className = '',
  ...props
}: ButtonProps) {
  return (
    <button
      className={`
        inline-flex items-center justify-center rounded-md font-semibold
        transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-offset-2
        disabled:opacity-50 disabled:cursor-not-allowed
        ${variantClasses[variant]}
        ${sizeClasses[size]}
        ${className}
      `}
      disabled={disabled || loading}
      {...props}
    >
      {loading && (
        <svg
          className="mr-2 h-4 w-4 animate-spin"
          viewBox="0 0 24 24"
          fill="none"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
          />
        </svg>
      )}
      {children}
    </button>
  );
}
