import { useEffect, useState } from 'react';
import { ChevronRight, Cpu, Database, Eye, Layers, Sparkles, FolderSearch, Search, Activity, Box, type LucideIcon } from 'lucide-react';
import { useStore, type Screen } from '../lib/storeContext';
import { Badge, Kbd, Logo, Panel, type Tone } from '../components/ui';
import { cx } from '../lib/cx';

const ACTIONS: { id: Screen; label: string; hint: string; icon: LucideIcon }[] = [
  { id: 'scan', label: 'Scan a directory', hint: 'Index files and find duplicates', icon: FolderSearch },
  { id: 'analyze', label: 'Analyze files with AI', hint: 'Describe, tag and name images', icon: Sparkles },
  { id: 'organize', label: 'Organize (rename & categorize)', hint: 'Review and apply the plan', icon: Layers },
  { id: 'search', label: 'Search files (semantic)', hint: 'Find images by meaning', icon: Search },
  { id: 'status', label: 'View status', hint: 'Models, index and history', icon: Activity },
];

export function HomeScreen() {
  const { overview, go } = useStore();
  const [active, setActive] = useState(0);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).closest('input, textarea')) return;
      if (e.key === 'ArrowDown') setActive((a) => (a + 1) % ACTIONS.length);
      else if (e.key === 'ArrowUp') setActive((a) => (a - 1 + ACTIONS.length) % ACTIONS.length);
      else if (e.key === 'Enter') go(ACTIONS[active].id);
      else return;
      e.preventDefault();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [active, go]);

  const vision = overview?.models.find((m) => m.kind === 'Vision');
  const embed = overview?.models.find((m) => m.kind === 'Embedding');
  const modelBadge = (installed: boolean | undefined): [Tone, string] =>
    !overview ? ['muted', 'Checking'] : !overview.online ? ['bad', 'Offline'] : installed ? ['ok', 'Ready'] : ['warn', 'Missing'];

  const rows: { icon: LucideIcon; label: string; value: string; badge: [Tone, string] }[] = [
    {
      icon: Cpu,
      label: 'Runtime',
      value: 'Ollama',
      badge: !overview ? ['muted', 'Checking'] : overview.online ? ['ok', 'Online'] : ['bad', 'Offline'],
    },
    { icon: Eye, label: 'Vision', value: vision?.name ?? '—', badge: modelBadge(vision?.installed) },
    { icon: Box, label: 'Embedding', value: embed?.name ?? '—', badge: modelBadge(embed?.installed) },
    {
      icon: Database,
      label: 'Storage',
      value: overview ? `JSON · ${overview.files.toLocaleString()} files` : 'JSON',
      badge: ['ok', 'Ready'],
    },
  ];

  return (
    <div className="animate-in mx-auto max-w-[720px] pt-4">
      <div className="mb-8 flex items-start justify-between">
        <div className="flex items-center gap-5">
          <Logo size={56} />
          <div>
            <div className="bg-gradient-to-r from-accent via-teal to-purple bg-clip-text text-[52px] font-extrabold leading-none tracking-tight text-transparent">
              zorter
            </div>
            <div className="mt-2 text-[13px] font-semibold tracking-[0.18em] text-accent">LOCAL AI FILE INTELLIGENCE</div>
            <div className="mt-1 text-[13px] text-dim">Understand. Organize. Discover.</div>
          </div>
        </div>
        <span className="pt-2 text-[13px] text-faint">v{overview?.version ?? ''}</span>
      </div>

      <Panel className="mb-6 px-5 py-4">
        <div className="space-y-2.5">
          {rows.map(({ icon: Icon, label, value, badge }) => (
            <div key={label} className="grid grid-cols-[22px_110px_1fr_auto] items-center gap-2 text-[14px]">
              <Icon size={16} className="text-dim" />
              <span className="text-dim">{label}</span>
              <span className="truncate">{value}</span>
              <Badge tone={badge[0]}>{badge[1]}</Badge>
            </div>
          ))}
        </div>
      </Panel>

      {overview && !overview.online && (
        <div className="mb-5 rounded-lg border border-bad/40 bg-bad/5 px-4 py-2.5 text-[13px] text-dim">
          Ollama isn't running. Start it (run <span className="text-text">ollama serve</span>) to analyze and search.
        </div>
      )}

      <nav className="space-y-1" aria-label="Actions">
        {ACTIONS.map(({ id, label, hint, icon: Icon }, i) => (
          <button
            key={id}
            onMouseEnter={() => setActive(i)}
            onClick={() => go(id)}
            className={cx(
              'group flex h-11 w-full items-center gap-3 rounded-lg px-3 text-left transition-colors',
              i === active ? 'bg-select text-white' : 'text-text hover:bg-raised',
            )}
          >
            <ChevronRight size={16} className={cx(i === active ? 'opacity-100' : 'opacity-0')} />
            <Icon size={17} className={i === active ? 'text-white' : 'text-accent'} />
            <span className="flex-1 text-[14px]">{label}</span>
            <span className={cx('text-[12px]', i === active ? 'text-white/70' : 'text-faint')}>{hint}</span>
          </button>
        ))}
      </nav>

      <div className="mt-6 flex items-center gap-5 text-[12px] text-dim">
        <span className="flex items-center gap-2">
          <Kbd>↑↓</Kbd> Navigate
        </span>
        <span className="flex items-center gap-2">
          <Kbd>Enter</Kbd> Select
        </span>
      </div>
    </div>
  );
}
