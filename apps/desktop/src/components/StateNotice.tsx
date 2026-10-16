import type { ReactNode } from "react";

export function LoadingNotice({ label = "Loading…" }: { label?: string }) {
  return (
    <p className="notice" role="status">
      {label}
    </p>
  );
}

export function EmptyNotice({ title, hint }: { title: string; hint?: ReactNode }) {
  return (
    <div className="notice" role="status">
      <p className="notice-title">{title}</p>
      {hint ? <p className="notice-hint">{hint}</p> : null}
    </div>
  );
}

export function ErrorNotice({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="notice notice-error" role="alert">
      <p className="notice-title">Something went wrong</p>
      <p className="notice-hint">{message}</p>
      {onRetry ? (
        <button type="button" onClick={onRetry}>
          Retry
        </button>
      ) : null}
    </div>
  );
}

export function UnsupportedNotice({ feature }: { feature: string }) {
  return (
    <div className="notice notice-muted" role="note">
      <p className="notice-title">{feature} is not supported</p>
      <p className="notice-hint">
        This device or driver does not declare this capability, so the controls are hidden.
      </p>
    </div>
  );
}
