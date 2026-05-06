interface SkeletonProps {
  variant?: 'text' | 'card' | 'circle' | 'rect';
  width?: string | number;
  height?: string | number;
  className?: string;
}

const variantClasses: Record<string, string> = {
  text: 'h-4 rounded-sm',
  card: 'h-32 rounded-lg',
  circle: 'rounded-full',
  rect: 'rounded-md',
};

export function Skeleton({ variant = 'text', width, height, className }: SkeletonProps) {
  return (
    <div
      className={`animate-pulse bg-surface-strong ${variantClasses[variant]} ${className || ''}`}
      style={{ width, height }}
      aria-hidden="true"
    />
  );
}
