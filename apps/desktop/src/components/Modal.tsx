import { useEffect } from "react";
import type { ReactNode } from "react";

type Props = {
  title: string;
  onClose: () => void;
  children: ReactNode;
};

/// A centered dialog with a dimmed backdrop — the app-wide pattern for
/// "create/add a new thing" (never for destructive confirmations, which
/// stay inline via `.confirm-box`). Closes on a backdrop click, on Escape,
/// or its own × button; the caller decides when to render it (no `open`
/// prop) and when to close it (no internal open state).
export function Modal({ title, onClose, children }: Props) {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{title}</h3>
          <button className="modal-close" onClick={onClose} aria-label="Close">
            ×
          </button>
        </div>
        <div className="modal-body">{children}</div>
      </div>
    </div>
  );
}
