import { Button } from '@/components/ui';
import { IconWarning } from '@/components/icons';

interface LinkWarningProps {
  link: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function LinkWarning({ link, onConfirm, onCancel }: LinkWarningProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink/50 backdrop-blur-sm">
      <div className="mx-4 w-full max-w-sm rounded-lg bg-canvas p-6 shadow-xl">
        <div className="flex items-center gap-3 mb-4">
          <IconWarning size={24} className="text-warning" />
          <h2 className="text-lg font-semibold text-ink">
            Open External Link
          </h2>
        </div>

        <p className="mb-2 text-sm text-body">
          You are about to open an external link. Please verify the URL before proceeding.
        </p>

        <div className="mb-4 rounded-md bg-surface-soft p-3">
          <p className="break-all text-sm font-mono text-ink">{link}</p>
        </div>

        <div className="flex gap-3">
          <Button variant="secondary" onClick={onCancel} className="flex-1">
            Cancel
          </Button>
          <Button variant="primary" onClick={onConfirm} className="flex-1">
            Open Link
          </Button>
        </div>
      </div>
    </div>
  );
}
