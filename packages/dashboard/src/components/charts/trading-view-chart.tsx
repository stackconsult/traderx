"use client";

import { useEffect, useRef } from "react";
import { useMarketDataStore } from "@/store";
import { useChartData } from "@/hooks/use-chart-data";
import { TimeframeSelector } from "./timeframe-selector";
import { cn } from "@/lib/utils";

interface TradingViewChartProps {
  className?: string;
}

export function TradingViewChart({ className }: TradingViewChartProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<ReturnType<typeof import("lightweight-charts").createChart> | null>(null);
  const candleSeriesRef = useRef<ReturnType<InstanceType<typeof import("lightweight-charts")["IChartApi"]>["addCandlestickSeries"]> | null>(null);

  const symbol = useMarketDataStore((state) => state.selectedSymbol);
  const { candles } = useChartData(symbol);

  useEffect(() => {
    if (!chartRef.current) return;

    let chart: ReturnType<typeof import("lightweight-charts").createChart> | null = null;

    const initChart = async () => {
      const { createChart, ColorType } = await import("lightweight-charts");

      chart = createChart(chartRef.current!, {
        layout: {
          background: { type: ColorType.Solid, color: "transparent" },
          textColor: "hsl(var(--foreground))",
        },
        grid: {
          vertLines: { color: "hsl(var(--border))" },
          horzLines: { color: "hsl(var(--border))" },
        },
        crosshair: {
          mode: 1,
        },
        rightPriceScale: {
          borderColor: "hsl(var(--border))",
        },
        timeScale: {
          borderColor: "hsl(var(--border))",
          timeVisible: true,
          secondsVisible: false,
        },
        width: chartRef.current!.clientWidth,
        height: chartRef.current!.clientHeight,
      });

      const candleSeries = chart.addCandlestickSeries({
        upColor: "#00C805",
        downColor: "#FF5000",
        borderUpColor: "#00C805",
        borderDownColor: "#FF5000",
        wickUpColor: "#00C805",
        wickDownColor: "#FF5000",
      });

      chartInstanceRef.current = chart;
      candleSeriesRef.current = candleSeries;
    };

    initChart();

    const handleResize = () => {
      if (chartInstanceRef.current && chartRef.current) {
        chartInstanceRef.current.resize(
          chartRef.current.clientWidth,
          chartRef.current.clientHeight
        );
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      if (chartInstanceRef.current) {
        chartInstanceRef.current.remove();
        chartInstanceRef.current = null;
        candleSeriesRef.current = null;
      }
    };
  }, []);

  // Update data when candles change
  useEffect(() => {
    if (!candleSeriesRef.current || candles.length === 0) return;
    candleSeriesRef.current.setData(
      candles.map((c) => ({
        time: c.time as number,
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
      }))
    );
  }, [candles]);

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Chart Controls */}
      <div className="flex items-center justify-between border-b px-4 py-2">
        <div className="flex items-center gap-3">
          <span className="text-sm font-semibold">{symbol}</span>
          <TimeframeSelector />
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>Candles</span>
        </div>
      </div>

      {/* Chart Container */}
      <div ref={chartRef} className="flex-1 w-full" />
    </div>
  );
}
