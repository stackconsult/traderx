"use client";

import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Search, TrendingUp, TrendingDown } from "lucide-react";
import { cn, formatCurrency } from "@/lib/utils";
import { useMarketDataStore } from "@/store";

// Mock symbol data for search
const POPULAR_SYMBOLS = [
  { symbol: "AAPL", name: "Apple Inc.", price: 178.35, change: 1.39 },
  { symbol: "GOOGL", name: "Alphabet Inc.", price: 142.8, change: -0.64 },
  { symbol: "MSFT", name: "Microsoft Corp.", price: 378.91, change: 1.37 },
  { symbol: "TSLA", name: "Tesla Inc.", price: 245.67, change: -1.29 },
  { symbol: "NVDA", name: "NVIDIA Corp.", price: 892.45, change: 1.40 },
  { symbol: "AMZN", name: "Amazon.com Inc.", price: 178.22, change: 0.85 },
  { symbol: "META", name: "Meta Platforms", price: 505.75, change: 2.12 },
  { symbol: "AMD", name: "AMD Inc.", price: 178.90, change: -0.45 },
];

export function SymbolSearch() {
  const [query, setQuery] = useState("");
  const [isOpen, setIsOpen] = useState(false);
  const addToWatchlist = useMarketDataStore((state) => state.addToWatchlist);
  const setSelectedSymbol = useMarketDataStore((state) => state.setSelectedSymbol);

  const filtered = query.length > 0
    ? POPULAR_SYMBOLS.filter(
        (s) =>
          s.symbol.toLowerCase().includes(query.toLowerCase()) ||
          s.name.toLowerCase().includes(query.toLowerCase())
      )
    : [];

  const handleSelect = (symbol: string) => {
    addToWatchlist(symbol);
    setSelectedSymbol(symbol);
    setQuery("");
    setIsOpen(false);
  };

  return (
    <div className="relative">
      <div className="relative">
        <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder="Search symbol..."
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setIsOpen(true);
          }}
          onFocus={() => setIsOpen(true)}
          className="pl-8"
        />
      </div>

      {isOpen && filtered.length > 0 && (
        <div className="absolute z-50 mt-1 w-full rounded-md border bg-popover shadow-lg">
          <ul className="max-h-60 overflow-auto py-1">
            {filtered.map((stock) => {
              const isPositive = stock.change >= 0;
              return (
                <li
                  key={stock.symbol}
                  className={cn(
                    "flex cursor-pointer items-center justify-between px-3 py-2 hover:bg-accent",
                    "text-sm"
                  )}
                  onClick={() => handleSelect(stock.symbol)}
                >
                  <div>
                    <div className="font-medium">{stock.symbol}</div>
                    <div className="text-xs text-muted-foreground">{stock.name}</div>
                  </div>
                  <div className="text-right">
                    <div className="font-medium">{formatCurrency(stock.price)}</div>
                    <div
                      className={cn(
                        "flex items-center text-xs",
                        isPositive ? "text-up" : "text-down"
                      )}
                    >
                      {isPositive ? (
                        <TrendingUp className="mr-1 h-3 w-3" />
                      ) : (
                        <TrendingDown className="mr-1 h-3 w-3" />
                      )}
                      {isPositive ? "+" : ""}
                      {stock.change}%
                    </div>
                  </div>
                </li>
              );
            })}
          </ul>
        </div>
      )}
    </div>
  );
}
