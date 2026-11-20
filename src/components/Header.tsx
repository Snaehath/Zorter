import React from 'react';
import {
  FolderKanban,
  Compass,
  LayoutDashboard,
  Settings,
  RotateCcw,
  Cpu,
  Sparkles,
} from 'lucide-react';

interface HeaderProps {
  currentTab: 'dashboard' | 'organize' | 'explore';
  onSelectTab: (tab: 'dashboard' | 'organize' | 'explore') => void;
  onOpenSettings: () => void;
  onUndoLast: () => void;
  isUndoAvailable: boolean;
  visionModel: string;
  embeddingModel: string;
  isProcessing: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  currentTab,
  onSelectTab,
  onOpenSettings,
  onUndoLast,
  isUndoAvailable,
  visionModel,
  embeddingModel,
  isProcessing,
}) => {
  return (
    <header className="border-b border-slate-800 bg-slate-900/80 backdrop-blur sticky top-0 z-40 px-6 py-3 flex items-center justify-between">
      {/* Brand */}
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-indigo-600 via-indigo-500 to-cyan-400 flex items-center justify-center shadow-lg shadow-indigo-500/20">
          <Sparkles className="w-5 h-5 text-white" />
        </div>
        <div>
          <div className="flex items-center gap-2">
            <span className="font-bold text-lg tracking-tight text-white">Zorter</span>
            <span className="text-[10px] uppercase font-semibold px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-400 border border-indigo-500/30">
              Local AI
            </span>
          </div>
          <p className="text-xs text-slate-400">Privacy-First AI File Organizer</p>
        </div>
      </div>

      {/* Navigation Tabs */}
      <nav className="flex items-center gap-1 bg-slate-950/60 p-1 rounded-xl border border-slate-800">
        <button
          onClick={() => onSelectTab('dashboard')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-medium transition-all ${
            currentTab === 'dashboard'
              ? 'bg-indigo-600 text-white shadow'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
          }`}
        >
          <LayoutDashboard className="w-4 h-4" />
          Dashboard
        </button>

        <button
          onClick={() => onSelectTab('organize')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-medium transition-all ${
            currentTab === 'organize'
              ? 'bg-indigo-600 text-white shadow'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
          }`}
        >
          <FolderKanban className="w-4 h-4" />
          Analyze & Organize
        </button>

        <button
          onClick={() => onSelectTab('explore')}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-medium transition-all ${
            currentTab === 'explore'
              ? 'bg-indigo-600 text-white shadow'
              : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
          }`}
        >
          <Compass className="w-4 h-4" />
          Explore & Search
        </button>
      </nav>

      {/* Model Status & Actions */}
      <div className="flex items-center gap-3">
        <div className="hidden lg:flex items-center gap-2 px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 text-xs">
          <Cpu className="w-3.5 h-3.5 text-cyan-400" />
          <span className="text-slate-400">Vision:</span>
          <span className="font-mono text-cyan-300 font-medium max-w-[120px] truncate" title={visionModel}>
            {visionModel}
          </span>
          <span className="text-slate-600">|</span>
          <span className="text-slate-400">Embed:</span>
          <span className="font-mono text-indigo-300 font-medium max-w-[120px] truncate" title={embeddingModel}>
            {embeddingModel}
          </span>
        </div>

        {/* Undo Button */}
        <button
          onClick={onUndoLast}
          disabled={!isUndoAvailable || isProcessing}
          title="Roll back the last file organization operation safely"
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition border ${
            isUndoAvailable && !isProcessing
              ? 'bg-amber-500/10 text-amber-300 border-amber-500/30 hover:bg-amber-500/20'
              : 'bg-slate-900 text-slate-600 border-slate-800 cursor-not-allowed'
          }`}
        >
          <RotateCcw className="w-3.5 h-3.5" />
          Undo
        </button>

        {/* Settings Button */}
        <button
          onClick={onOpenSettings}
          className="p-2 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 border border-slate-800 transition"
          title="Settings"
        >
          <Settings className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
