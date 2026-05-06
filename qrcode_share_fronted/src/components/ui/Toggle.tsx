interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  description?: string;
  disabled?: boolean;
  variant?: 'default' | 'danger';
}

export function Toggle({ checked, onChange, label, description, disabled, variant = 'default' }: ToggleProps) {
  const trackColor = checked
    ? variant === 'danger' ? 'bg-error' : 'bg-ink'
    : 'bg-hairline';

  return (
    <label className={`flex items-center justify-between rounded-md bg-surface-soft p-3 ${disabled ? 'opacity-50' : 'cursor-pointer'}`}>
      <div>
        <p className="text-sm font-medium text-ink">{label}</p>
        {description && <p className="text-xs text-muted">{description}</p>}
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => !disabled && onChange(!checked)}
        className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200 ${trackColor}`}
      >
        <span
          className={`inline-block h-4 w-4 transform rounded-full bg-on-primary transition-transform duration-200 ${
            checked ? 'translate-x-6' : 'translate-x-1'
          }`}
        />
      </button>
    </label>
  );
}
