// Remembers how long analysis takes per image on this machine, so the
// Analyze screen can show time estimates before a run.

const KEY = 'zorter.msPerImage';

export function recordAnalysisSpeed(analyzed: number, elapsedMs: number): void {
  if (analyzed < 1 || elapsedMs <= 0) return;
  const current = elapsedMs / analyzed;
  const previous = msPerImage();
  // Smooth across runs so one unusually fast/slow run doesn't dominate.
  const next = previous ? previous * 0.5 + current * 0.5 : current;
  try {
    localStorage.setItem(KEY, String(Math.round(next)));
  } catch {
    /* storage unavailable: no estimates */
  }
}

export function msPerImage(): number | null {
  try {
    const v = Number(localStorage.getItem(KEY));
    return Number.isFinite(v) && v > 0 ? v : null;
  } catch {
    return null;
  }
}

/** Human estimate for `n` images, e.g. "≈ 3 min", or null when unknown. */
export function estimateFor(n: number): string | null {
  const ms = msPerImage();
  if (!ms || n < 1) return null;
  const secs = (ms * n) / 1000;
  if (secs < 60) return '< 1 min';
  if (secs < 3600) return `≈ ${Math.round(secs / 60)} min`;
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  return `≈ ${h} h ${m} min`;
}
