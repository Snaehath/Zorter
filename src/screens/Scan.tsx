import { ArrowRight, CircleCheck, Copy, File, FileText, FolderSearch, Image, Plus, RefreshCw, Files, type LucideIcon } from 'lucide-react';
import { useStore } from '../lib/storeContext';
import { bytes, duration, plural } from '../lib/format';
import { FolderBar } from '../components/FolderBar';
import { Badge, Button, Heading, Notice, Panel, SegmentBar } from '../components/ui';

function StatRow({ icon: Icon, label, value, tone }: { icon: LucideIcon; label: string; value: number; tone?: string }) {
  return (
    <div className="flex items-center gap-3 py-1.5">
      <Icon size={16} className="text-dim" />
      <span className="flex-1 text-dim">{label}</span>
      <span className={tone ?? 'text-text'}>{value.toLocaleString()}</span>
    </div>
  );
}

export function ScanScreen() {
  const { folder, scan, startScan, go, overview } = useStore();
  const r = scan.report;
  const p = scan.progress;
  const ratio = r ? 1 : p && p.total > 0 ? p.current / p.total : 0;

  return (
    <div className="animate-in">
      <Heading icon={FolderSearch} title="Scan" />
      <FolderBar disabled={scan.running}>
        <Button variant="primary" icon={FolderSearch} onClick={() => void startScan()} disabled={!folder} busy={scan.running}>
          {r ? 'Scan again' : 'Scan'}
        </Button>
      </FolderBar>

      {scan.error && <Notice tone="bad" title="Scan failed">{scan.error}</Notice>}

      {!scan.running && !r && !scan.error && (
        <Panel className="px-6 py-10 text-center text-dim">
          {folder ? 'Press Scan to index this folder.' : 'Choose a folder to get started.'}
        </Panel>
      )}

      {(scan.running || r) && (
        <Panel className="animate-in px-6 py-5">
          <div className="grid grid-cols-[110px_1fr] gap-y-1.5 text-[14px]">
            <span className="text-dim">Directory</span>
            <span className="truncate">{r?.directory ?? folder}</span>
            <span className="text-dim">Mode</span>
            <span>Recursive</span>
            <span className="text-dim">AI</span>
            <span>
              {overview?.online ? <Badge tone="ok">Enabled</Badge> : <Badge tone="warn">Offline — scan only</Badge>}
            </span>
          </div>

          <div className="my-5 border-t border-line-soft" />

          <div className="mb-2 text-dim">{scan.running ? 'Scanning files…' : 'Scan finished'}</div>
          <div className="flex items-center gap-4">
            <SegmentBar value={ratio} indeterminate={scan.running && !p} />
            <span className="w-12 text-right font-semibold">{Math.round(ratio * 100)}%</span>
          </div>
          {scan.running && p && (
            <div className="mt-2 text-[12px] text-faint">
              {p.current.toLocaleString()} / {p.total.toLocaleString()} images
            </div>
          )}

          {r && (
            <>
              <div className="mt-6 grid grid-cols-2 gap-x-10 border-b border-line-soft pb-5">
                <div className="border-r border-line-soft pr-10">
                  <StatRow icon={Files} label="Files discovered" value={r.files_discovered} />
                  <StatRow icon={Image} label="Images" value={r.images} />
                  <StatRow icon={FileText} label="Documents" value={r.documents} />
                  <StatRow icon={File} label="Other" value={r.other} />
                </div>
                <div>
                  <StatRow icon={RefreshCw} label="Already indexed" value={r.already_indexed} tone="text-accent" />
                  <StatRow icon={Plus} label="New files" value={r.new_files} tone="text-ok" />
                  <StatRow icon={Copy} label="Duplicates" value={r.duplicates} tone={r.duplicates ? 'text-warn' : 'text-dim'} />
                </div>
              </div>

              <div className="mt-5 flex items-start justify-between gap-6">
                <div>
                  <div className="mb-2 flex items-center gap-2 font-semibold text-ok">
                    <CircleCheck size={18} /> Scan completed
                  </div>
                  <ul className="space-y-1 pl-7 text-[13px] text-dim">
                    <li>
                      {plural(r.files_discovered, 'file')} discovered ({plural(r.images, 'image')})
                    </li>
                    <li>
                      <span className={r.needs_analysis ? 'text-warn' : 'text-ok'}>{r.needs_analysis.toLocaleString()}</span> files
                      require analysis
                    </li>
                    <li>{plural(r.already_indexed, 'file')} already indexed</li>
                    <li>
                      {plural(r.duplicates, 'duplicate candidate')}
                      {r.wasted_bytes > 0 && ` (${bytes(r.wasted_bytes)} reclaimable)`}
                    </li>
                    <li className="text-faint">Finished in {duration(r.elapsed_ms)}</li>
                  </ul>
                </div>
                <Button
                  variant="primary"
                  icon={ArrowRight}
                  onClick={() => go(r.needs_analysis > 0 ? 'analyze' : 'organize')}
                  className="mt-1"
                >
                  {r.needs_analysis > 0 ? 'Next: Analyze' : 'Next: Organize'}
                </Button>
              </div>
            </>
          )}
        </Panel>
      )}
    </div>
  );
}
