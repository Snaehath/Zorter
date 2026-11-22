import { useEffect, useState, type ReactNode } from 'react';
import { ArrowRight, Box, CircleCheck, CircleX, Clock, Cpu, File, FileText, Play, RotateCcw, Sparkles, Square } from 'lucide-react';
import { useStore } from '../lib/storeContext';
import { duration, plural } from '../lib/format';
import { api, errorText } from '../lib/api';
import { estimateFor } from '../lib/estimate';
import type { AnalyzeFileEvent, AnalyzePreview } from '../lib/types';
import { FolderBar } from '../components/FolderBar';
import { Button, Heading, Notice, Panel, SegmentBar, Tag, Thumb } from '../components/ui';
import { cx } from '../lib/cx';

function useNow(active: boolean) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!active) return;
    const id = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(id);
  }, [active]);
  return now;
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="grid grid-cols-[100px_1fr] gap-2 text-[13px]">
      <span className="text-dim">{label}</span>
      <span className="truncate text-text" title={typeof children === 'string' ? children : undefined}>
        {children || '—'}
      </span>
    </div>
  );
}

function CurrentFile({ e }: { e: AnalyzeFileEvent }) {
  const a = e.analysis!;
  const tags = [...a.tags, ...a.subject].filter((t, i, all) => t && all.indexOf(t) === i).slice(0, 8);
  return (
    <Panel className="animate-in p-5">
      <div className="flex gap-6">
        <Thumb key={e.file_id} path={e.path} hash={e.hash} className="h-[170px] w-[240px] shrink-0" />
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-3">
            <span className="text-[12px] font-semibold tracking-wider text-accent">CURRENT FILE</span>
            <span className="text-[12px] text-dim">
              ({e.index} / {e.total})
            </span>
          </div>
          <div className="mt-0.5 truncate text-[15px] font-semibold" title={e.name}>
            {e.name}
          </div>

          <div className="mt-3 text-[12px] font-semibold text-purple">Description</div>
          <p className="mt-0.5 line-clamp-3 text-[13px] leading-relaxed">{a.description}</p>

          <div className="mt-3 grid grid-cols-2 gap-x-6 gap-y-1">
            <Field label="Category">{a.category}</Field>
            <Field label="Style">{a.visual_style}</Field>
            <Field label="Subcategory">{a.subcategory}</Field>
            <Field label="Lighting">{a.lighting ?? ''}</Field>
            <Field label="Environment">{a.environment ?? ''}</Field>
            <Field label="Colors">{a.colors.slice(0, 3).join(', ')}</Field>
          </div>
        </div>
      </div>

      {tags.length > 0 && (
        <div className="mt-4">
          <div className="mb-1.5 text-[12px] font-semibold text-purple">Tags</div>
          <div className="flex flex-wrap gap-1.5">
            {tags.map((t) => (
              <Tag key={t}>{t.toLowerCase().replace(/\s+/g, '-')}</Tag>
            ))}
          </div>
        </div>
      )}

      {e.proposed_name && (
        <div className="mt-4">
          <div className="mb-1.5 text-[12px] font-semibold text-purple">Suggested name</div>
          <span className="inline-flex max-w-full items-center gap-2 rounded-lg border border-line bg-raised px-3 py-1.5 text-[13px]">
            <FileText size={14} className="shrink-0 text-dim" />
            <span className="truncate text-ok">{e.proposed_name}</span>
          </span>
        </div>
      )}
    </Panel>
  );
}

function FeedRow({ e }: { e: AnalyzeFileEvent }) {
  return (
    <div className="flex items-center gap-3 border-b border-line-soft px-4 py-2 text-[13px] last:border-b-0">
      <Thumb path={e.path} hash={e.hash} className="h-8 w-11 shrink-0 rounded-md" />
      <span className="w-[32%] truncate text-dim" title={e.name}>
        {e.name}
      </span>
      {e.error ? (
        <span className="flex min-w-0 flex-1 items-center gap-2 text-bad">
          <CircleX size={14} className="shrink-0" />
          <span className="truncate" title={e.error}>
            {e.error}
          </span>
        </span>
      ) : (
        <>
          <ArrowRight size={14} className="shrink-0 text-faint" />
          <span className="min-w-0 flex-1 truncate text-ok" title={e.proposed_name ?? ''}>
            {e.proposed_name}
          </span>
          <span className="max-w-[28%] truncate text-accent">
            {e.analysis?.category} / {e.analysis?.subcategory}
          </span>
        </>
      )}
    </div>
  );
}

/** Decide what to analyze: only what's left (default) or everything again. */
function StartPanel({
  preview,
  onStart,
}: {
  preview: { data: AnalyzePreview | null; error: string | null } | null;
  onStart: (force: boolean) => void;
}) {
  if (!preview) {
    return <Panel className="mb-5 animate-pulse px-5 py-4 text-[13px] text-dim">Checking this folder…</Panel>;
  }
  if (preview.error || !preview.data) {
    return (
      <div className="mb-5">
        <Notice tone="bad" title="Couldn't read this folder">
          {preview.error}
        </Notice>
      </div>
    );
  }
  const { total, analyzed, remaining } = preview.data;
  if (total === 0) {
    return <Panel className="mb-5 px-5 py-4 text-[13px] text-dim">No supported images (PNG, JPG, WebP, BMP) in this folder.</Panel>;
  }
  const hint = (n: number) => estimateFor(n);
  const label = (text: string, n: number) => (hint(n) ? `${text} · ${hint(n)}` : text);

  return (
    <Panel className="animate-in mb-5 flex flex-wrap items-center gap-4 px-5 py-4">
      <div className="min-w-[220px] flex-1">
        {remaining === 0 ? (
          <div className="flex items-center gap-2 font-semibold text-ok">
            <CircleCheck size={17} /> All {plural(total, 'image')} are analyzed
          </div>
        ) : analyzed === 0 ? (
          <div className="font-semibold">{plural(total, 'image')} to analyze</div>
        ) : (
          <div className="font-semibold">
            {remaining.toLocaleString()} of {plural(total, 'image')} left to analyze
          </div>
        )}
        <div className="mt-0.5 text-[12px] text-dim">
          {remaining === 0
            ? 'Nothing to do. Re-analyze only if you changed the vision model or want fresh results.'
            : analyzed > 0
              ? `${analyzed.toLocaleString()} already have results from an earlier run — skipping them saves time and GPU.`
              : 'Each image is described, tagged and named by your local vision model.'}
        </div>
      </div>
      <div className="flex flex-wrap gap-2">
        {remaining > 0 && (
          <Button variant="primary" icon={Play} onClick={() => onStart(false)}>
            {label(analyzed > 0 ? `Analyze remaining ${remaining.toLocaleString()}` : `Analyze ${total.toLocaleString()}`, remaining)}
          </Button>
        )}
        {analyzed > 0 && (
          <Button icon={RotateCcw} onClick={() => onStart(true)}>
            {label(`Re-analyze all ${total.toLocaleString()}`, total)}
          </Button>
        )}
      </div>
    </Panel>
  );
}

export function AnalyzeScreen() {
  const { folder, analyze: s, startAnalyze, cancelAnalyze, overview, go } = useStore();
  const now = useNow(s.running);
  const [preview, setPreview] = useState<{ dir: string; data: AnalyzePreview | null; error: string | null } | null>(null);

  // Count analyzed vs. remaining images whenever the folder changes or a run ends.
  useEffect(() => {
    if (!folder || s.running) return;
    let alive = true;
    api.analyzePreview(folder).then(
      (data) => alive && setPreview({ dir: folder, data, error: null }),
      (e: unknown) => alive && setPreview({ dir: folder, data: null, error: errorText(e) }),
    );
    return () => {
      alive = false;
    };
  }, [folder, s.running]);
  const current = preview?.dir === folder ? preview : null;

  const total = s.started?.total ?? 0;
  const ratio = total > 0 ? s.processed / total : s.summary ? 1 : 0;
  const elapsed = s.summary ? s.summary.elapsed_ms : s.startedAt ? now - s.startedAt : 0;
  const vision = overview?.models.find((m) => m.kind === 'Vision');
  const searchModel = s.started?.search_model ?? overview?.models.find((m) => m.kind === 'Embedding')?.name;
  const hasRun = s.running || s.summary || s.error || s.feed.length > 0;

  return (
    <div className="animate-in">
      <Heading icon={Sparkles} title="AI Analysis" />

      <FolderBar disabled={s.running}>
        {s.running && (
          <Button variant="danger" icon={Square} onClick={() => void cancelAnalyze()} busy={s.cancelling}>
            {s.cancelling ? 'Stopping…' : 'Stop'}
          </Button>
        )}
      </FolderBar>

      {folder && !s.running && <StartPanel preview={current} onStart={(force) => void startAnalyze(force)} />}

      <Panel className="mb-5 grid grid-cols-2 divide-x divide-line-soft px-1 py-3 text-[13px]">
        <div className="space-y-1.5 px-5">
          <div className="flex items-center gap-2.5">
            <Box size={15} className="text-dim" />
            <span className="text-dim">Model:</span>
            <span className="truncate font-semibold">{s.started?.model ?? vision?.name ?? '—'}</span>
          </div>
          <div className="flex items-center gap-2.5">
            <File size={15} className="text-dim" />
            <span className="text-dim">Processing:</span>
            <span>{total ? `${Math.min(s.processed + (s.running ? 1 : 0), total)} / ${total}` : '—'}</span>
          </div>
        </div>
        <div className="space-y-1.5 px-5">
          <div className="flex items-center gap-2.5">
            <Cpu size={15} className="text-dim" />
            <span className="text-dim">Runtime:</span>
            <span>Ollama</span>
            <span className="ml-auto truncate text-[12px] text-faint">search: {searchModel ?? 'off'}</span>
          </div>
          <div className="flex items-center gap-2.5">
            <Clock size={15} className="text-dim" />
            <span className="text-dim">Elapsed:</span>
            <span>{s.startedAt || s.summary ? duration(elapsed) : '—'}</span>
          </div>
        </div>
      </Panel>

      {s.error && (
        <div className="mb-5">
          <Notice tone="bad" title="Analysis couldn't run">
            {s.error}
          </Notice>
        </div>
      )}

      {!hasRun && !folder && <Panel className="px-6 py-10 text-center text-dim">Choose a folder to get started.</Panel>}

      {s.running && !s.started && (
        <Panel className="mb-5 px-6 py-5">
          <div className="mb-2 text-dim">Indexing folder…</div>
          <SegmentBar
            value={s.indexing && s.indexing.total ? s.indexing.current / s.indexing.total : 0}
            indeterminate={!s.indexing}
          />
        </Panel>
      )}

      {s.current && <CurrentFile e={s.current} />}
      {s.running && s.started && !s.current && (
        <Panel className="px-6 py-10 text-center text-dim">
          <Sparkles className="mx-auto mb-2 animate-pulse text-purple" />
          Analyzing the first image…
        </Panel>
      )}

      {(s.started || s.summary) && (
        <Panel className="mt-5 flex items-center gap-8 px-6 py-4">
          <div className="flex-1">
            <div className="mb-2 text-[13px] font-semibold text-purple">Overall progress</div>
            <div className="flex items-center gap-4">
              <SegmentBar value={ratio} color="purple" segments={40} />
              <span className="w-12 text-right font-semibold">{Math.round(ratio * 100)}%</span>
            </div>
          </div>
          <div className="grid grid-cols-[auto_auto] gap-x-4 gap-y-0.5 border-l border-line-soft pl-8 text-[13px]">
            <span className="text-dim">Analyzed:</span>
            <span className="text-right text-ok">
              {s.analyzed} / {total}
            </span>
            <span className="text-dim">Remaining:</span>
            <span className="text-right text-warn">{Math.max(0, total - s.processed)}</span>
            <span className="text-dim">Failed:</span>
            <span className={cx('text-right', s.failed ? 'text-bad' : 'text-dim')}>{s.failed}</span>
          </div>
        </Panel>
      )}

      {s.summary && (
        <div className="mt-5 flex items-center justify-between gap-4">
          {s.summary.cancelled ? (
            <Notice tone="warn" title="Stopped">
              Finished results are saved — press Analyze to resume.
            </Notice>
          ) : s.summary.total === 0 ? (
            <Notice tone="ok" title="Everything here is already analyzed">
              Use “Re-analyze all” above to run it again.
            </Notice>
          ) : (
            <Notice tone="ok" title={`Analyzed ${s.summary.analyzed} file${s.summary.analyzed === 1 ? '' : 's'}`}>
              {s.summary.search_indexed > 0 && `${s.summary.search_indexed} added to the search index. `}
              {s.summary.failed > 0 && `${s.summary.failed} failed — run again to retry them.`}
            </Notice>
          )}
          <Button variant="primary" icon={ArrowRight} onClick={() => go('organize')} className="shrink-0">
            Next: Organize
          </Button>
        </div>
      )}

      {s.feed.length > 0 && (
        <Panel className="mt-5 overflow-hidden">
          <div className="border-b border-line-soft px-4 py-2.5 text-[12px] font-semibold tracking-wider text-accent">
            PROCESSED ({s.feed.length})
          </div>
          <div className="max-h-[320px] overflow-y-auto">
            {s.feed.map((e) => (
              <FeedRow key={`${e.file_id}-${e.index}`} e={e} />
            ))}
          </div>
        </Panel>
      )}
    </div>
  );
}
