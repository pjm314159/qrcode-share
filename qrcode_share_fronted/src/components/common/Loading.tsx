export function Loading({ className = '' }: { className?: string }) {
  return (
    <div className={`flex justify-center py-8 ${className}`}>
      <div className="h-8 w-8 animate-spin rounded-full border-4 border-surface-card border-t-ink" />
    </div>
  );
}

export function LoadingOverlay({ className = '' }: { className?: string }) {
  return (
    <div className={`absolute inset-0 flex items-center justify-center bg-canvas/60 ${className}`}>
      <div className="h-8 w-8 animate-spin rounded-full border-4 border-surface-card border-t-ink" />
    </div>
  );
}
