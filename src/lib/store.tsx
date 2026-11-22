import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react';
import { api, errorText, events } from './api';
import type { Overview } from './types';
import { StoreContext, type AnalyzeState, type ScanState, type Screen, type Store } from './storeContext';

const idleScan: ScanState = { running: false, progress: null, report: null, error: null };
const idleAnalyze: AnalyzeState = {
  running: false,
  cancelling: false,
  indexing: null,
  started: null,
  startedAt: null,
  current: null,
  feed: [],
  processed: 0,
  analyzed: 0,
  failed: 0,
  summary: null,
  error: null,
};

const FOLDER_KEY = 'zorter.folder';
function loadFolder(): string | null {
  try {
    return localStorage.getItem(FOLDER_KEY);
  } catch {
    return null;
  }
}

export function StoreProvider({ children }: { children: ReactNode }) {
  const [screen, setScreen] = useState<Screen>('home');
  const [folder, setFolder] = useState<string | null>(loadFolder);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [overviewError, setOverviewError] = useState<string | null>(null);
  const [scan, setScan] = useState<ScanState>(idleScan);
  const [analyze, setAnalyze] = useState<AnalyzeState>(idleAnalyze);

  const refreshOverview = useCallback(async () => {
    try {
      setOverview(await api.overview());
      setOverviewError(null);
    } catch (e) {
      setOverviewError(errorText(e));
    }
  }, []);

  useEffect(() => {
    const id = window.setInterval(() => void refreshOverview(), 30_000);
    // First load: state is only set once the request resolves.
    api.overview().then(setOverview, (e: unknown) => setOverviewError(errorText(e)));
    return () => window.clearInterval(id);
  }, [refreshOverview]);

  // Job event listeners live for the whole session so progress keeps
  // updating while the user is on another screen.
  useEffect(() => {
    const subs = [
      events.scanProgress((p) => setScan((s) => ({ ...s, progress: p }))),
      events.analyzeIndexing((p) => setAnalyze((s) => ({ ...s, indexing: p }))),
      events.analyzeStarted((p) => setAnalyze((s) => ({ ...s, started: p, indexing: null, startedAt: Date.now() }))),
      events.analyzeFile((e) =>
        setAnalyze((s) => ({
          ...s,
          current: e.analysis ? e : s.current,
          feed: [e, ...s.feed].slice(0, 300),
          processed: s.processed + 1,
          analyzed: s.analyzed + (e.analysis ? 1 : 0),
          failed: s.failed + (e.error ? 1 : 0),
        })),
      ),
    ];
    return () => subs.forEach((p) => void p.then((un) => un()));
  }, []);

  const chooseFolder = useCallback(async () => {
    const picked = await api.pickDirectory();
    if (picked) {
      setFolder(picked);
      try {
        localStorage.setItem(FOLDER_KEY, picked);
      } catch {
        /* storage unavailable: folder just isn't remembered */
      }
      setScan(idleScan);
    }
    return picked;
  }, []);

  const startScan = useCallback(async () => {
    const dir = folder;
    if (!dir) return;
    setScan({ ...idleScan, running: true });
    try {
      const report = await api.scan(dir);
      setScan({ running: false, progress: null, report, error: null });
    } catch (e) {
      setScan({ ...idleScan, error: errorText(e) });
    }
    void refreshOverview();
  }, [folder, refreshOverview]);

  const startAnalyze = useCallback(
    async (force: boolean) => {
      const dir = folder;
      if (!dir) return;
      setAnalyze({ ...idleAnalyze, running: true });
      try {
        const summary = await api.analyze(dir, force);
        setAnalyze((s) => ({ ...s, running: false, cancelling: false, summary }));
      } catch (e) {
        setAnalyze((s) => ({ ...s, running: false, cancelling: false, error: errorText(e) }));
      }
      void refreshOverview();
    },
    [folder, refreshOverview],
  );

  const cancelAnalyze = useCallback(async () => {
    setAnalyze((s) => ({ ...s, cancelling: true }));
    await api.cancelAnalysis();
  }, []);

  const value = useMemo<Store>(
    () => ({
      screen,
      go: setScreen,
      folder,
      chooseFolder,
      overview,
      overviewError,
      refreshOverview,
      scan,
      startScan,
      analyze,
      startAnalyze,
      cancelAnalyze,
    }),
    [screen, folder, chooseFolder, overview, overviewError, refreshOverview, scan, startScan, analyze, startAnalyze, cancelAnalyze],
  );

  return <StoreContext.Provider value={value}>{children}</StoreContext.Provider>;
}
