import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  ArrowLeft,
  ArrowRight,
  Ban,
  CircleCheck,
  Download,
  Folder,
  FolderOpen,
  Layers,
  ListChecks,
  RefreshCw,
  TriangleExclamationPoint,
  Undo2,
  X,
  type LucideIcon,
} from 'lucide-react';
import { api, errorText } from '../lib/api';
import { useStore } from '../lib/storeContext';
import { baseName, plural } from '../lib/format';
import { loadOptions, saveOptions } from '../lib/planOptions';
import type { PlanOptions, PlanReport, PlannedChange } from '../lib/types';
import { FolderBar } from '../components/FolderBar';
import { OrganizeOptions } from '../components/OrganizeOptions';
import { Button, Chip, Heading, Notice, Panel, SectionLabel, Thumb } from '../components/ui';
import { cx } from '../lib/cx';

type Confirm = { changes: PlannedChange[]; label: string };

function ConfirmDialog({ c, busy, onCancel, onConfirm }: { c: Confirm; busy: boolean; onCancel: () => void; onConfirm: () => void }) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && !busy && onCancel();
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [busy, onCancel]);
  const uncertain = c.changes.filter((x) => x.uncertain).length;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6" role="dialog" aria-modal="true">
      <Panel className="animate-in w-full max-w-[440px] p-6">
        <div className="mb-1 text-[16px] font-semibold">{c.label}?</div>
        <p className="text-[13px] leading-relaxed text-dim">
          {plural(c.changes.length, 'file')} will be renamed or moved
          {uncertain > 0 && `, including ${uncertain} unanalyzed into Needs-Review/`}. Nothing is overwritten, and you can
          undo this at any time.
        </p>
        <div className="mt-5 flex justify-end gap-2">
          <Button onClick={onCancel} disabled={busy}>
            Cancel
          </Button>
          <Button variant="success" icon={CircleCheck} onClick={onConfirm} busy={busy} autoFocus>
            Apply {c.changes.length}
          </Button>
        </div>
      </Panel>
    </div>
  );
}

function ActionRow({
  icon: Icon,
  label,
  detail,
  onClick,
  tone,
  primary,
}: {
  icon: LucideIcon;
  label: string;
  detail?: string;
  onClick: () => void;
  tone?: string;
  primary?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={cx(
        'flex h-10 w-full items-center gap-3 rounded-lg px-3 text-left text-[14px] transition-colors',
        primary ? 'bg-ok/15 text-ok hover:bg-ok/25' : 'hover:bg-raised',
      )}
    >
      <Icon size={16} className={tone ?? (primary ? 'text-ok' : 'text-dim')} />
      <span className="flex-1">{label}</span>
      {detail && <span className="text-[12px] text-faint">{detail}</span>}
    </button>
  );
}

function Review({
  plan,
  onBack,
  onApply,
}: {
  plan: PlanReport;
  onBack: () => void;
  onApply: (changes: PlannedChange[]) => void;
}) {
  const [selected, setSelected] = useState<Set<string>>(() => new Set(plan.changes.filter((c) => !c.uncertain).map((c) => c.file_id)));
  const [filter, setFilter] = useState<'all' | 'ready' | 'uncertain'>('all');
  const rows = plan.changes.filter((c) => filter === 'all' || (filter === 'uncertain') === c.uncertain);
  const toggle = (id: string) =>
    setSelected((s) => {
      const n = new Set(s);
      if (n.has(id)) n.delete(id);
      else n.add(id);
      return n;
    });
  const setAll = (on: boolean) =>
    setSelected((s) => {
      const n = new Set(s);
      rows.forEach((r) => (on ? n.add(r.file_id) : n.delete(r.file_id)));
      return n;
    });
  const chosen = plan.changes.filter((c) => selected.has(c.file_id));

  return (
    <div className="animate-in">
      <div className="mb-3 flex items-center gap-2">
        <Button icon={ArrowLeft} onClick={onBack}>
          Back
        </Button>
        <div className="ml-2 flex rounded-lg border border-line p-0.5 text-[12px]">
          {(['all', 'ready', 'uncertain'] as const).map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={cx('rounded-md px-3 py-1 capitalize', filter === f ? 'bg-select text-white' : 'text-dim hover:text-text')}
            >
              {f === 'uncertain' ? 'Needs review' : f}
            </button>
          ))}
        </div>
        <div className="ml-auto flex gap-2">
          <Button onClick={() => setAll(true)}>Select all</Button>
          <Button onClick={() => setAll(false)}>Select none</Button>
        </div>
      </div>

      <Panel className="overflow-hidden">
        <div className="grid grid-cols-[36px_56px_1fr_20px_1fr_minmax(0,0.8fr)] items-center gap-3 border-b border-line-soft px-4 py-2 text-[12px] text-faint">
          <span />
          <span />
          <span>Original</span>
          <span />
          <span>Proposed</span>
          <span>Folder</span>
        </div>
        <div className="max-h-[calc(100vh-300px)] overflow-y-auto">
          {rows.map((c) => {
            const on = selected.has(c.file_id);
            return (
              <label
                key={c.file_id + c.from}
                className={cx(
                  'grid cursor-pointer grid-cols-[36px_56px_1fr_20px_1fr_minmax(0,0.8fr)] items-center gap-3 border-b border-line-soft px-4 py-2 text-[13px] last:border-b-0 hover:bg-raised',
                  !on && 'opacity-55',
                )}
                title={c.description || 'Not analyzed — goes to Needs-Review/'}
              >
                <input type="checkbox" checked={on} onChange={() => toggle(c.file_id)} className="h-4 w-4 accent-[#4ade80]" />
                <Thumb path={c.from} hash={c.hash} className="h-9 w-14 rounded-md" />
                <span className="truncate text-dim">{c.original_name}</span>
                <ArrowRight size={14} className="text-faint" />
                <span className={cx('truncate', c.uncertain ? 'text-warn' : 'text-ok', !on && 'line-through')}>{c.proposed_name}</span>
                <span className={cx('truncate', c.uncertain ? 'text-warn' : 'text-accent')}>{c.folder}/</span>
              </label>
            );
          })}
        </div>
      </Panel>

      <div className="mt-4 flex items-center justify-between">
        <span className="text-[13px] text-dim">
          {chosen.length} of {plan.changes.length} selected
        </span>
        <Button variant="success" icon={CircleCheck} disabled={chosen.length === 0} onClick={() => onApply(chosen)}>
          Apply {chosen.length} change{chosen.length === 1 ? '' : 's'}
        </Button>
      </div>
    </div>
  );
}

export function OrganizeScreen() {
  const { folder, go, refreshOverview, analyze } = useStore();
  const [plan, setPlan] = useState<PlanReport | null>(null);
  const [loading, setLoading] = useState(Boolean(folder));
  const [error, setError] = useState<string | null>(null);
  const [reviewing, setReviewing] = useState(false);
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [applying, setApplying] = useState(false);
  const [done, setDone] = useState<{ organized: number } | null>(null);
  const [undoing, setUndoing] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const [options, setOptions] = useState<Required<PlanOptions>>(loadOptions);

  const load = useCallback(async () => {
    if (!folder) return;
    setLoading(true);
    setError(null);
    try {
      setPlan(await api.plan(folder, options));
    } catch (e) {
      setError(errorText(e));
    } finally {
      setLoading(false);
    }
  }, [folder, options]);

  // (Re)build the plan whenever the options change. The screen is keyed by
  // folder, so choosing another folder remounts it.
  useEffect(() => {
    if (!folder) return;
    let alive = true;
    api
      .plan(folder, options)
      .then((p) => alive && setPlan(p), (e: unknown) => alive && setError(errorText(e)))
      .finally(() => alive && setLoading(false));
    return () => {
      alive = false;
    };
  }, [folder, options]);

  const changeOptions = (o: Required<PlanOptions>) => {
    saveOptions(o);
    setLoading(true);
    setError(null);
    setNote(null);
    setOptions(o);
  };

  const ready = useMemo(() => plan?.changes.filter((c) => !c.uncertain) ?? [], [plan]);

  const apply = async () => {
    if (!confirm) return;
    setApplying(true);
    try {
      const r = await api.apply(confirm.changes);
      setDone(r);
      setNote(null);
      setConfirm(null);
      setReviewing(false);
      await load();
    } catch (e) {
      setConfirm(null);
      setError(errorText(e));
    } finally {
      setApplying(false);
      void refreshOverview();
    }
  };

  const undo = async () => {
    setUndoing(true);
    try {
      const r = await api.undo();
      setDone(null);
      setNote(`Restored ${plural(r.restored, 'file')} to their original names and folders.`);
      await load();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setUndoing(false);
      void refreshOverview();
    }
  };

  const exportPlan = async () => {
    if (!plan) return;
    try {
      const path = await api.exportPlan(plan.changes, plan.root_name);
      if (path) setNote(`Plan exported to ${path}`);
    } catch (e) {
      setError(errorText(e));
    }
  };

  return (
    <div className="animate-in">
      <Heading icon={Layers} title="Organization plan">
        <Button icon={RefreshCw} onClick={() => void load()} disabled={!folder || analyze.running} busy={loading}>
          Refresh
        </Button>
      </Heading>
      <FolderBar disabled={loading || applying} />

      {analyze.running && (
        <div className="mb-4">
          <Notice tone="info" title="Analysis is running">
            The plan will include more files once it finishes — press Refresh then.
          </Notice>
        </div>
      )}
      {error && (
        <div className="mb-4">
          <Notice tone="bad" title="Something went wrong">
            {error}
          </Notice>
        </div>
      )}
      {done && (
        <div className="animate-in mb-4 flex items-center justify-between gap-4 rounded-xl border border-ok/50 bg-panel px-4 py-3">
          <div className="flex items-center gap-2 font-semibold text-ok">
            <CircleCheck size={18} /> Organized {plural(done.organized, 'file')}.
          </div>
          <Button icon={Undo2} onClick={() => void undo()} busy={undoing}>
            Undo
          </Button>
        </div>
      )}
      {note && (
        <div className="mb-4">
          <Notice tone="info" title={note} />
        </div>
      )}

      {!folder && <Panel className="px-6 py-10 text-center text-dim">Choose a folder to plan its organization.</Panel>}
      {folder && loading && !plan && <Panel className="animate-pulse px-6 py-10 text-center text-dim">Building plan…</Panel>}

      {plan && plan.analyzed === 0 && (
        <Panel className="px-6 py-8 text-center">
          <div className="mb-1 text-warn">No files in this folder have been analyzed yet.</div>
          <div className="mb-4 text-[13px] text-dim">Zorter needs AI descriptions before it can suggest names and folders.</div>
          <Button variant="primary" icon={ArrowRight} onClick={() => go('analyze')}>
            Analyze this folder
          </Button>
        </Panel>
      )}

      {folder && !reviewing && (!plan || plan.analyzed > 0) && (
        <OrganizeOptions value={options} root={plan?.root_name ?? baseName(folder)} disabled={applying} onChange={changeOptions} />
      )}

      {plan && plan.analyzed > 0 && reviewing && (
        <Review key={plan.changes.length + options.layout + options.naming} plan={plan} onBack={() => setReviewing(false)} onApply={(changes) => setConfirm({ changes, label: 'Apply selected changes' })} />
      )}

      {plan && plan.analyzed > 0 && !reviewing && (
        <div className={cx('transition-opacity', loading && 'opacity-50')} aria-busy={loading}>
          <div className="mb-5 flex flex-wrap gap-3 text-[13px]">
            <Chip icon={CircleCheck} tone="ok">
              {plural(plan.analyzed, 'file')} analyzed
            </Chip>
            <Chip icon={CircleCheck} tone="ok">
              {plan.ready} ready
            </Chip>
            {plan.review > 0 && (
              <Chip icon={TriangleExclamationPoint} tone="warn">
                {plan.review} need review
              </Chip>
            )}
            {plan.unchanged > 0 && (
              <Chip tone="muted">
                <span className="text-dim">{plan.unchanged} already match</span>
              </Chip>
            )}
            {plan.skipped_unanalyzed > 0 && (
              <Chip tone="muted">
                <span className="text-dim">{plan.skipped_unanalyzed} unanalyzed left in place</span>
              </Chip>
            )}
          </div>

          {plan.changes.length === 0 ? (
            <Panel className="px-6 py-8 text-center text-ok">
              <CircleCheck className="mx-auto mb-2" /> Nothing to change — every file already matches these options.
            </Panel>
          ) : (
            <>
              <Panel className="grid grid-cols-[minmax(240px,0.8fr)_1.4fr] divide-x divide-line-soft">
                <div className="p-5">
                  <SectionLabel className="mb-3">PROPOSED STRUCTURE</SectionLabel>
                  <div className="flex items-center gap-2 text-[14px]">
                    <FolderOpen size={16} className="text-accent" />
                    <span className="truncate font-semibold">{plan.root_name}/</span>
                  </div>
                  <ul className="ml-[7px] mt-1 border-l border-line pl-4">
                    {plan.folders.slice(0, 9).map((f) => (
                      <li key={f.name} className="flex items-center gap-2 py-1 text-[13px]">
                        <Folder size={15} className={f.name === 'Needs-Review' ? 'text-warn' : 'text-accent'} />
                        <span className={cx('flex-1 truncate', f.name === 'Needs-Review' && 'text-warn')}>
                          {f.name === '.' ? '(renamed in place)' : `${f.name}/`}
                        </span>
                        <span className="text-faint">({f.count})</span>
                      </li>
                    ))}
                    {plan.folders.length > 9 && <li className="py-1 text-[12px] text-faint">… {plan.folders.length - 9} more folders</li>}
                  </ul>
                </div>
                <div className="min-w-0 p-5">
                  <SectionLabel className="mb-3">
                    PREVIEW <span className="font-normal text-faint">(first {Math.min(5, plan.changes.length)} of {plan.changes.length})</span>
                  </SectionLabel>
                  <div className="grid grid-cols-[minmax(0,1fr)_18px_minmax(0,1.3fr)] gap-x-3 gap-y-2 text-[13px]">
                    <span className="border-b border-line-soft pb-1.5 text-dim">Original</span>
                    <span className="border-b border-line-soft" />
                    <span className="border-b border-line-soft pb-1.5 text-dim">Proposed</span>
                    {plan.changes.slice(0, 5).map((c) => (
                      <div key={c.file_id + c.from} className="contents">
                        <span className="truncate" title={c.original_name}>
                          {c.original_name}
                        </span>
                        <ArrowRight size={14} className="mt-0.5 text-faint" />
                        <span className={cx('truncate', c.uncertain ? 'text-warn' : 'text-ok')} title={`${c.folder}/${c.proposed_name}`}>
                          {c.proposed_name}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              </Panel>

              <Panel className="mt-5 space-y-0.5 p-2">
                <ActionRow
                  icon={FolderOpen}
                  primary
                  label="Apply all suggested changes"
                  detail={plural(plan.changes.length, 'file')}
                  onClick={() => setConfirm({ changes: plan.changes, label: 'Apply all suggested changes' })}
                />
                <ActionRow icon={ListChecks} label="Review individually" detail="Pick which files to change" onClick={() => setReviewing(true)} />
                {plan.review > 0 && (
                  <ActionRow
                    icon={Ban}
                    label="Exclude uncertain files"
                    detail={`Apply ${plural(ready.length, 'ready file')}`}
                    onClick={() => setConfirm({ changes: ready, label: 'Apply changes to ready files' })}
                  />
                )}
                <ActionRow icon={Download} label="Export organization plan (JSON)" onClick={() => void exportPlan()} />
                <ActionRow icon={X} label="Cancel" tone="text-bad" onClick={() => go('home')} />
              </Panel>
            </>
          )}
        </div>
      )}

      {confirm && <ConfirmDialog c={confirm} busy={applying} onCancel={() => setConfirm(null)} onConfirm={() => void apply()} />}
    </div>
  );
}
