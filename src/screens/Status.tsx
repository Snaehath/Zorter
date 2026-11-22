import { useEffect, useState, type ReactNode } from 'react';
import { Activity, Box, CircleCheck, Clock, Cpu, Database, Eye, FileText, RefreshCw, Undo2, type LucideIcon } from 'lucide-react';
import { api, errorText } from '../lib/api';
import { useStore } from '../lib/storeContext';
import { bytes, plural, relativeTime } from '../lib/format';
import { Badge, Button, Heading, Notice, Panel, SectionLabel } from '../components/ui';

const COLS = 'grid grid-cols-[150px_minmax(0,1fr)_100px_120px] gap-3 px-5';

function Row({ icon: Icon, label, children }: { icon: LucideIcon; label: string; children: ReactNode }) {
  return (
    <div className="grid grid-cols-[22px_150px_1fr] items-center gap-2 py-1.5 text-[14px]">
      <Icon size={16} className="text-dim" />
      <span className="text-dim">{label}</span>
      <span className="min-w-0 truncate">{children}</span>
    </div>
  );
}

export function StatusScreen() {
  const { overview: o, overviewError, refreshOverview, analyze } = useStore();
  const [switching, setSwitching] = useState(false);
  const analyzing = analyze.running;
  const [refreshing, setRefreshing] = useState(false);
  const [undoing, setUndoing] = useState(false);
  const [message, setMessage] = useState<{ tone: 'ok' | 'bad'; text: string } | null>(null);

  useEffect(() => {
    void refreshOverview();
  }, [refreshOverview]);

  const refresh = async () => {
    setRefreshing(true);
    await refreshOverview();
    setRefreshing(false);
  };

  const undo = async () => {
    setUndoing(true);
    try {
      const r = await api.undo();
      setMessage({ tone: 'ok', text: `Restored ${plural(r.restored, 'file')} to their original names and folders.` });
    } catch (e) {
      setMessage({ tone: 'bad', text: errorText(e) });
    } finally {
      setUndoing(false);
      void refreshOverview();
    }
  };

  const chooseVision = async (model: string) => {
    if (model === o?.models.find((m) => m.kind === 'Vision')?.name) return;
    setSwitching(true);
    try {
      await api.setVisionModel(model);
      setMessage({ tone: 'ok', text: `Now using ${model} for image analysis. Existing results are kept; use “Re-analyze all” to refresh them.` });
    } catch (e) {
      setMessage({ tone: 'bad', text: errorText(e) });
    } finally {
      setSwitching(false);
      void refreshOverview();
    }
  };

  const ready = o?.models.filter((m) => m.installed).length ?? 0;

  return (
    <div className="animate-in">
      <Heading icon={Activity} title="Status">
        <Button icon={RefreshCw} onClick={() => void refresh()} busy={refreshing}>
          Refresh
        </Button>
      </Heading>

      {overviewError && <Notice tone="bad" title="Couldn't load status">{overviewError}</Notice>}

      <SectionLabel className="mb-2">MODELS</SectionLabel>
      <Panel className="mb-6 overflow-hidden">
        <div className={`${COLS} border-b border-line-soft py-2.5 text-[12px] text-faint`}>
          <span>TYPE</span>
          <span>MODEL</span>
          <span>SIZE</span>
          <span>STATUS</span>
        </div>
        {o &&
          o.models.map((m) => {
            const isVision = m.kind === 'Vision';
            return (
              <div key={m.kind} className={`${COLS} items-center border-b border-line-soft py-3 text-[14px] last:border-b-0`}>
                <span className="flex items-center gap-2.5">
                  {isVision ? <Eye size={16} className="text-accent" /> : <Box size={16} className="text-accent" />}
                  {m.kind}
                </span>
                {isVision ? (
                  <select
                    value={m.name}
                    onChange={(e) => void chooseVision(e.target.value)}
                    disabled={switching || analyzing}
                    title={analyzing ? 'Wait for the current analysis to finish' : 'Vision model used for analysis'}
                    className="w-fit max-w-full cursor-pointer truncate rounded-md border border-line bg-panel px-2.5 py-1 text-[14px] text-text outline-none focus:border-accent disabled:cursor-not-allowed disabled:opacity-60"
                  >
                    {o.vision_models.map((v, i) => (
                      <option key={v.name} value={v.name}>
                        {v.name} ({i === 0 ? 'default' : 'backup'}){o.online && !v.installed ? ' · not installed' : ''}
                      </option>
                    ))}
                  </select>
                ) : (
                  <span className="truncate">{m.name}</span>
                )}
                <span className="text-dim">{m.size_bytes ? bytes(m.size_bytes) : '—'}</span>
                <span>
                  {!o.online ? <Badge tone="bad">Offline</Badge> : m.installed ? <Badge tone="ok">Ready</Badge> : <Badge tone="warn">Missing</Badge>}
                </span>
              </div>
            );
          })}
        {!o && <div className="animate-pulse px-5 py-4 text-dim">Checking…</div>}
      </Panel>
      {o?.online &&
        o.models
          .filter((m) => !m.installed)
          .map((m) => (
            <div key={m.name} className="-mt-4 mb-6 text-[13px] text-dim">
              Install {m.name} with <span className="text-text">ollama pull {m.name}</span>
              {m.kind === 'Vision' && ', or choose another vision model above'}
            </div>
          ))}

      <SectionLabel className="mb-2">STATUS</SectionLabel>
      <Panel className="mb-6 px-5 py-3">
        {o ? (
          <>
            <Row icon={FileText} label="Index">
              {plural(o.files, 'file')}{' '}
              <span className="text-dim">
                · {o.analyzed.toLocaleString()} analyzed · {o.searchable.toLocaleString()} searchable
              </span>
            </Row>
            <Row icon={Database} label="Storage">
              JSON{' '}
              <span className="text-dim" title={o.storage_path}>
                ({bytes(o.storage_bytes)}, {o.storage_path})
              </span>
            </Row>
            <Row icon={Clock} label="Last scan">
              {relativeTime(o.last_scan)}
            </Row>
            <Row icon={Clock} label="Last analysis">
              {relativeTime(o.last_analysis)}
            </Row>
            <Row icon={Cpu} label="Runtime">
              <span className="flex items-center gap-3">
                <Badge tone={o.online ? 'ok' : 'bad'}>{o.online ? 'Online' : 'Offline'}</Badge>
                <span className="text-dim">{o.endpoint}</span>
              </span>
            </Row>
            <Row icon={Box} label="Models">
              {o.online ? `${ready} of ${o.models.length} ready` : 'Unavailable'}
            </Row>
            <Row icon={CircleCheck} label="Operations">
              {o.operations_completed} completed{' '}
              <span className="text-dim">
                ({o.operations_failed} failed, {o.operations_undone} undone)
              </span>
            </Row>
          </>
        ) : (
          <div className="animate-pulse py-2 text-dim">Loading…</div>
        )}
      </Panel>

      <SectionLabel className="mb-2">LAST ORGANIZATION</SectionLabel>
      <Panel className="flex items-center justify-between gap-4 px-5 py-4">
        {o?.undoable ? (
          <>
            <span className="text-[14px]">
              {plural(o.undoable.files, 'file')} renamed <span className="text-dim">· {relativeTime(o.undoable.timestamp)}</span>
            </span>
            <Button icon={Undo2} onClick={() => void undo()} busy={undoing}>
              Undo
            </Button>
          </>
        ) : (
          <span className="text-dim">Nothing to undo.</span>
        )}
      </Panel>
      {message && (
        <div className="mt-4">
          <Notice tone={message.tone} title={message.text} />
        </div>
      )}
    </div>
  );
}
