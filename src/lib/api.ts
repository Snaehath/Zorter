import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AnalyzeFileEvent,
  AnalyzeStarted,
  AnalyzeSummary,
  Overview,
  PlanReport,
  PlannedChange,
  Progress,
  ScanReport,
  SearchResponse,
} from './types';

export const api = {
  overview: () => invoke<Overview>('get_overview'),
  pickDirectory: () => invoke<string | null>('pick_directory'),
  thumbnail: (path: string, hash: string) => invoke<string | null>('get_thumbnail', { path, hash }),
  reveal: (path: string) => invoke<void>('reveal_in_folder', { path }),

  scan: (dir: string) => invoke<ScanReport>('scan_folder', { dir }),

  analyze: (dir: string, force: boolean) => invoke<AnalyzeSummary>('analyze_folder', { dir, force }),
  cancelAnalysis: () => invoke<boolean>('cancel_analysis'),

  plan: (dir: string) => invoke<PlanReport>('plan_folder', { dir }),
  apply: (changes: PlannedChange[]) => invoke<{ organized: number }>('apply_changes', { changes }),
  exportPlan: (changes: PlannedChange[], rootName: string) =>
    invoke<string | null>('export_plan', { changes, rootName }),
  undo: () => invoke<{ restored: number }>('undo_last'),

  search: (query: string, limit: number) => invoke<SearchResponse>('search_files', { query, limit }),
};

export const events = {
  scanProgress: (cb: (p: Progress) => void): Promise<UnlistenFn> =>
    listen<Progress>('scan:progress', (e) => cb(e.payload)),
  analyzeIndexing: (cb: (p: Progress) => void): Promise<UnlistenFn> =>
    listen<Progress>('analyze:indexing', (e) => cb(e.payload)),
  analyzeStarted: (cb: (p: AnalyzeStarted) => void): Promise<UnlistenFn> =>
    listen<AnalyzeStarted>('analyze:started', (e) => cb(e.payload)),
  analyzeFile: (cb: (p: AnalyzeFileEvent) => void): Promise<UnlistenFn> =>
    listen<AnalyzeFileEvent>('analyze:file', (e) => cb(e.payload)),
};

/** Errors from Rust arrive as plain strings. */
export function errorText(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  return String(e);
}
