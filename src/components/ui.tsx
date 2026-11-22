import { useEffect, useState, type ButtonHTMLAttributes, type ReactNode } from 'react';
import { ImageIcon, LoaderCircle, type LucideIcon } from 'lucide-react';
import { api } from '../lib/api';
import { cx } from '../lib/cx';

/** Nested-diamond brand mark. */
export function Logo({ size = 28 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
      <defs>
        <linearGradient id="zg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#22d3ee" />
          <stop offset="1" stopColor="#a78bfa" />
        </linearGradient>
      </defs>
      <rect x="6" y="6" width="20" height="20" rx="3" transform="rotate(45 16 16)" fill="none" stroke="url(#zg)" strokeWidth="2.6" />
      <rect x="12" y="12" width="8" height="8" rx="1.5" transform="rotate(45 16 16)" fill="url(#zg)" />
    </svg>
  );
}

export function Heading({ icon: Icon, title, children }: { icon: LucideIcon; title: string; children?: ReactNode }) {
  return (
    <div className="mb-5 flex items-center justify-between gap-4">
      <h1 className="flex items-center gap-2.5 text-[15px] font-semibold tracking-wide text-accent">
        <Icon size={18} strokeWidth={2.2} />
        <span>
          ZORTER <span className="text-faint">/</span> {title.toUpperCase()}
        </span>
      </h1>
      {children && <div className="flex items-center gap-2">{children}</div>}
    </div>
  );
}

export function Panel({
  children,
  className,
  tone = 'default',
}: {
  children: ReactNode;
  className?: string;
  tone?: 'default' | 'ok' | 'accent';
}) {
  return (
    <section
      className={cx(
        'rounded-xl border bg-panel',
        tone === 'ok' && 'border-ok/60 shadow-[0_0_0_1px_rgba(74,222,128,0.15)]',
        tone === 'accent' && 'border-accent/50',
        tone === 'default' && 'border-line',
        className,
      )}
    >
      {children}
    </section>
  );
}

export function SectionLabel({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cx('text-[12px] font-semibold tracking-wider text-accent', className)}>{children}</div>;
}

export type Tone = 'ok' | 'warn' | 'bad' | 'info' | 'muted';
const toneText: Record<Tone, string> = {
  ok: 'text-ok',
  warn: 'text-warn',
  bad: 'text-bad',
  info: 'text-accent',
  muted: 'text-dim',
};

export function Badge({ tone, children }: { tone: Tone; children: ReactNode }) {
  return (
    <span className={cx('inline-flex items-center gap-1.5 whitespace-nowrap', toneText[tone])}>
      <span className={cx('h-2 w-2 rounded-full bg-current', tone === 'muted' && 'animate-pulse')} />
      {children}
    </span>
  );
}

/** Segmented progress bar, like the terminal mockup. */
export function SegmentBar({
  value,
  color = 'accent',
  segments = 32,
  indeterminate = false,
}: {
  value: number;
  color?: 'accent' | 'purple';
  segments?: number;
  indeterminate?: boolean;
}) {
  const filled = Math.round(Math.min(1, Math.max(0, value)) * segments);
  const on = color === 'purple' ? 'bg-purple' : 'bg-accent';
  return (
    <div
      className="flex h-3.5 flex-1 gap-[3px]"
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(value * 100)}
    >
      {Array.from({ length: segments }, (_, i) => (
        <div
          key={i}
          className={cx(
            'flex-1 rounded-[3px] transition-colors duration-300',
            indeterminate ? 'animate-pulse bg-line' : i < filled ? on : 'bg-line-soft',
          )}
          style={indeterminate ? { animationDelay: `${(i % 8) * 90}ms` } : undefined}
        />
      ))}
    </div>
  );
}

export function Button({
  variant = 'ghost',
  icon: Icon,
  busy,
  children,
  className,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'ghost' | 'danger' | 'success';
  icon?: LucideIcon;
  busy?: boolean;
}) {
  return (
    <button
      {...rest}
      disabled={rest.disabled || busy}
      className={cx(
        'inline-flex h-9 items-center justify-center gap-2 rounded-lg px-3.5 text-[13px] font-medium transition-colors',
        'disabled:cursor-not-allowed disabled:opacity-50',
        variant === 'primary' && 'bg-accent text-bg hover:bg-accent/85',
        variant === 'success' && 'bg-ok text-bg hover:bg-ok/85',
        variant === 'danger' && 'border border-bad/50 text-bad hover:bg-bad/10',
        variant === 'ghost' && 'border border-line text-text hover:border-accent/60 hover:bg-raised',
        className,
      )}
    >
      {busy ? <LoaderCircle size={15} className="animate-spin" /> : Icon && <Icon size={15} />}
      {children}
    </button>
  );
}

export function Chip({ icon: Icon, tone, children }: { icon?: LucideIcon; tone: Tone; children: ReactNode }) {
  return (
    <span className={cx('inline-flex items-center gap-2 rounded-lg border border-line bg-raised px-3 py-1.5', toneText[tone])}>
      {Icon && <Icon size={15} />}
      <span className="text-text">{children}</span>
    </span>
  );
}

export function Tag({ children }: { children: ReactNode }) {
  return (
    <span className="rounded-md border border-teal/40 bg-teal/10 px-2 py-0.5 text-[12px] text-teal">#{children}</span>
  );
}

export function Notice({ tone, title, children }: { tone: 'bad' | 'warn' | 'ok' | 'info'; title: string; children?: ReactNode }) {
  const border = { bad: 'border-bad/50', warn: 'border-warn/50', ok: 'border-ok/50', info: 'border-accent/50' }[tone];
  return (
    <div className={cx('animate-in rounded-xl border bg-panel px-4 py-3', border)}>
      <div className={cx('font-semibold', toneText[tone])}>{title}</div>
      {children && <div className="mt-1 text-[13px] text-dim">{children}</div>}
    </div>
  );
}

export function Kbd({ children }: { children: ReactNode }) {
  return <kbd className="rounded-md border border-line bg-raised px-1.5 py-0.5 font-mono text-[12px] text-text">{children}</kbd>;
}

// ---------------------------------------------------------------------------
// Thumbnails
// ---------------------------------------------------------------------------

const thumbCache = new Map<string, Promise<string | null>>();

function loadThumb(path: string, hash: string): Promise<string | null> {
  const key = `${hash}|${path}`;
  let p = thumbCache.get(key);
  if (!p) {
    p = api.thumbnail(path, hash).catch(() => null);
    thumbCache.set(key, p);
  }
  return p;
}

export function Thumb({ path, hash, className }: { path: string; hash: string; className?: string }) {
  const key = `${hash}|${path}`;
  const [loaded, setLoaded] = useState<{ key: string; src: string | null } | null>(null);
  useEffect(() => {
    let alive = true;
    void loadThumb(path, hash).then((src) => alive && setLoaded({ key, src }));
    return () => {
      alive = false;
    };
  }, [path, hash, key]);
  // `undefined` while loading (including right after the image changes).
  const src = loaded?.key === key ? loaded.src : undefined;
  return (
    <div className={cx('overflow-hidden rounded-lg border border-line bg-raised', className)}>
      {src ? (
        <img src={src} alt="" className="h-full w-full object-cover" draggable={false} />
      ) : (
        <div className={cx('flex h-full w-full items-center justify-center text-faint', src === undefined && 'animate-pulse')}>
          <ImageIcon size={20} />
        </div>
      )}
    </div>
  );
}
