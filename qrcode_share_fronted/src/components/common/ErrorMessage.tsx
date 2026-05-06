interface ErrorMessageProps {
  message: string;
  onDismiss?: () => void;
}

export function ErrorMessage({ message, onDismiss }: ErrorMessageProps) {
  return (
    <div className="rounded-md bg-error/10 p-3 text-sm text-error">
      <p>{message}</p>
      {onDismiss && (
        <button onClick={onDismiss} className="mt-1 underline">
          Dismiss
        </button>
      )}
    </div>
  );
}
