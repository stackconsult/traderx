"use client";

import { useState, useEffect, useRef } from "react";

type FlashDirection = "up" | "down" | null;

export function usePriceFlash(currentPrice: number) {
  const [flash, setFlash] = useState<FlashDirection>(null);
  const prevPrice = useRef(currentPrice);

  useEffect(() => {
    if (prevPrice.current === currentPrice) return;

    const direction = currentPrice > prevPrice.current ? "up" : "down";
    setFlash(direction);

    const timer = setTimeout(() => {
      setFlash(null);
    }, 300);

    prevPrice.current = currentPrice;

    return () => clearTimeout(timer);
  }, [currentPrice]);

  return flash;
}
