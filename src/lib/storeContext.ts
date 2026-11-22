import { createContext, useContext } from 'react';
import type { AnalyzeFileEvent, AnalyzeStarted, AnalyzeSummary, Overview, Progress, ScanReport } from './types';

export type Screen = 'home' | 'scan' | 'analyze' | 'organize' | 'search' | 'status';

export interface ScanState {
  running: boolean;
  progress: Progress | null;
  report: ScanReport | null;
  error: string | null;
}

export interface AnalyzeState {
  running: boolean;
  cancelling: boolean;
  indexing: Progress | null;
  started: AnalyzeStarted | null;
  startedAt: number | null;
  /** Most recent successfully analyzed file (drives the "current file" card). */
  current: AnalyzeFileEvent | null;
  /** Processed files, newest first. */
  feed: AnalyzeFileEvent[];
  processed: number;
  analyzed: number;
  failed: number;
  summary: AnalyzeSummary | null;
  error: string | null;
}

export interface Store {
  screen: Screen;
  go: (s: Screen) => void;
  folder: string | null;
  chooseFolder: () => Promise<string | null>;
  overview: Overview | null;
  overviewError: string | null;
  refreshOverview: () => Promise<void>;
  scan: ScanState;
  startScan: () => Promise<void>;
  analyze: AnalyzeState;
  startAnalyze: (force: boolean) => Promise<void>;
  cancelAnalyze: () => Promise<void>;
}

export const StoreContext = createContext<Store | null>(null);

export function useStore(): Store {
  const s = useContext(StoreContext);
  if (!s) throw new Error('useStore must be used inside <StoreProvider>');
  return s;
}
