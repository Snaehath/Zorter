import { Activity, FolderSearch, House, Layers, Search, Sparkles, type LucideIcon } from 'lucide-react';
import { StoreProvider } from './lib/store';
import { useStore, type Screen } from './lib/storeContext';
import { Badge, Logo } from './components/ui';
import { cx } from './lib/cx';
import { HomeScreen } from './screens/Home';
import { ScanScreen } from './screens/Scan';
import { AnalyzeScreen } from './screens/Analyze';
import { OrganizeScreen } from './screens/Organize';
import { SearchScreen } from './screens/Search';
import { StatusScreen } from './screens/Status';

const NAV: { id: Screen; label: string; icon: LucideIcon }[] = [
  { id: 'home', label: 'Home', icon: House },
  { id: 'scan', label: 'Scan', icon: FolderSearch },
  { id: 'analyze', label: 'Analyze', icon: Sparkles },
  { id: 'organize', label: 'Organize', icon: Layers },
  { id: 'search', label: 'Search', icon: Search },
  { id: 'status', label: 'Status', icon: Activity },
];

function Sidebar() {
  const { screen, go, overview, analyze, scan } = useStore();
  const busy: Partial<Record<Screen, boolean>> = { analyze: analyze.running, scan: scan.running };
  return (
    <aside className="flex w-[212px] shrink-0 flex-col border-r border-line-soft bg-panel/60">
      <div className="flex items-center gap-2.5 px-5 pb-6 pt-6">
        <Logo size={26} />
        <span className="bg-gradient-to-r from-accent to-purple bg-clip-text text-[20px] font-bold tracking-tight text-transparent">
          zorter
        </span>
      </div>
      <nav className="flex flex-col gap-1 px-3" aria-label="Main">
        {NAV.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => go(id)}
            aria-current={screen === id ? 'page' : undefined}
            className={cx(
              'flex h-10 items-center gap-3 rounded-lg px-3 text-left text-[13px] transition-colors',
              screen === id ? 'bg-select text-white' : 'text-dim hover:bg-raised hover:text-text',
            )}
          >
            <Icon size={17} />
            <span className="flex-1">{label}</span>
            {busy[id] && <span className="h-2 w-2 animate-pulse rounded-full bg-purple" title="Running" />}
          </button>
        ))}
      </nav>
      <div className="mt-auto space-y-1.5 border-t border-line-soft px-5 py-4 text-[12px]">
        <div className="flex items-center justify-between">
          <span className="text-faint">Ollama</span>
          {overview ? (
            <Badge tone={overview.online ? 'ok' : 'bad'}>{overview.online ? 'Online' : 'Offline'}</Badge>
          ) : (
            <Badge tone="muted">Checking</Badge>
          )}
        </div>
        <div className="text-faint">v{overview?.version ?? '…'}</div>
      </div>
    </aside>
  );
}

function Current() {
  const { screen, folder } = useStore();
  switch (screen) {
    case 'home':
      return <HomeScreen />;
    case 'scan':
      return <ScanScreen />;
    case 'analyze':
      return <AnalyzeScreen />;
    case 'organize':
      return <OrganizeScreen key={folder ?? 'none'} />;
    case 'search':
      return <SearchScreen />;
    case 'status':
      return <StatusScreen />;
  }
}

export default function App() {
  return (
    <StoreProvider>
      <div className="flex h-full">
        <Sidebar />
        <main className="min-w-0 flex-1 overflow-y-auto">
          <div className="mx-auto max-w-[1080px] px-8 py-7">
            <Current />
          </div>
        </main>
      </div>
    </StoreProvider>
  );
}
