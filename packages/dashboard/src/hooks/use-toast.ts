"use client";

import { useState, useCallback } from "react";
import type { ToastVariant } from "@/components/ui/toast";

interface ToastMessage {
  id: string;
  title: string;
  description?: string;
  variant?: ToastVariant;
}

let globalToastFn: ((toast: Omit<ToastMessage, "id">) => void) | null = null;

export function useToast() {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const addToast = useCallback((toast: Omit<ToastMessage, "id">) => {
    const id = Math.random().toString(36).slice(2);
    setToasts((prev) => [...prev, { ...toast, id }]);
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4000);
  }, []);

  globalToastFn = addToast;

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return { toasts, toast: addToast, dismiss };
}

export function toast(message: Omit<ToastMessage, "id">) {
  if (globalToastFn) {
    globalToastFn(message);
  }
}
