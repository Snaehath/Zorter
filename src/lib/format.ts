export function bytes(n: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let v = n;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return i === 0 ? `${n} B` : `${v.toFixed(1)} ${units[i]}`;
}

export function duration(ms: number): string {
  const s = Math.floor(ms / 1000);
  if (ms < 10_000) return `${(ms / 1000).toFixed(1)}s`;
  if (s < 60) return `${s}s`;
  if (s < 3600) return `${Math.floor(s / 60)}m ${String(s % 60).padStart(2, '0')}s`;
  return `${Math.floor(s / 3600)}h ${String(Math.floor((s % 3600) / 60)).padStart(2, '0')}m`;
}

export function relativeTime(iso: string | null | undefined): string {
  if (!iso) return 'Never';
  const secs = Math.max(0, (Date.now() - new Date(iso).getTime()) / 1000);
  if (secs < 45) return 'Just now';
  const [n, unit] =
    secs < 3600 ? [Math.round(secs / 60), 'minute'] : secs < 86400 ? [Math.floor(secs / 3600), 'hour'] : [Math.floor(secs / 86400), 'day'];
  const v = Math.max(1, n);
  return `${v} ${unit}${v === 1 ? '' : 's'} ago`;
}

export function plural(n: number, word: string, many = `${word}s`): string {
  return `${n.toLocaleString()} ${n === 1 ? word : many}`;
}

/** Last path segment, works for both slash styles. */
export function baseName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path;
}
