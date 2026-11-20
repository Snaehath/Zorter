import React from 'react';
import {
  FolderOpen,
  Image as ImageIcon,
  Brain,
  Search,
  Copy,
  RotateCcw,
  CheckCircle2,
  Clock,
  HardDrive,
  Cpu,
  Layers,
  ArrowRight,
  Square,
} from 'lucide-react';
import type { AppSettings, AppStats, IndexedFile, OperationJournalEntry } from '../types';

interface DashboardScreenProps {
  stats: AppStats | null;
  settings: AppSettings | null;
  recentFiles: IndexedFile[];
  recentHistory: OperationJournalEntry[];
  onSelectDirectory: () => void;
  onNavigateTab: (tab: 'dashboard' | 'organize' | 'explore') => void;
  onUndoLast: () => void;
  isScanning: boolean;
  scanProgress: { current: number; total: number; current_file: string } | null;
  isAnalyzing: boolean;
  analyzeProgress: { current: number; total: number; file_name: string; status: string } | null;
  onCancelAnalysis?: () => void;
}

export const DashboardScreen: React.FC<DashboardScreenProps> = ({
  stats,
  settings,
  recentFiles,
  recentHistory,
  onSelectDirectory,
  onNavigateTab,
  onUndoLast,
  isScanning,
  scanProgress,
  isAnalyzing,
  analyzeProgress,
  onCancelAnalysis,
}) => {
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  };

  return (
    <div className="space-y-8 max-w-7xl mx-auto pb-12">
      {/* Real-time Task Progress Banner */}
      {isScanning && scanProgress && (
        <div className="p-4 rounded-2xl bg-indigo-950/60 border border-indigo-500/40 shadow-xl shadow-indigo-950/50 flex flex-col gap-2 animate-pulse">
          <div className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-2 text-indigo-300 font-medium">
              <FolderOpen className="w-4 h-4 text-indigo-400" />
              <span>Scanning Directory: {scanProgress.current_file}</span>
            </div>
            <span className="font-mono text-indigo-200">
              {scanProgress.current} / {scanProgress.total} files
            </span>
          </div>
          <div className="w-full bg-slate-900 rounded-full h-2 overflow-hidden">
            <div
              className="bg-indigo-500 h-2 rounded-full transition-all duration-200"
              style={{
                width: `${scanProgress.total > 0 ? (scanProgress.current / scanProgress.total) * 100 : 0}%`,
              }}
            />
          </div>
        </div>
      )}

      {isAnalyzing && analyzeProgress && (
        <div className="p-4 rounded-2xl bg-cyan-950/70 border border-cyan-500/40 shadow-xl shadow-cyan-950/50 flex flex-col gap-3">
          <div className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-2 text-cyan-300 font-medium truncate max-w-[70%]">
              <Brain className="w-4 h-4 text-cyan-400 shrink-0 animate-pulse" />
              <span className="truncate">{analyzeProgress.status}: <strong className="text-white">{analyzeProgress.file_name}</strong></span>
            </div>
            <div className="flex items-center gap-3 shrink-0">
              <span className="font-mono text-cyan-200">
                {analyzeProgress.current} / {analyzeProgress.total} ({analyzeProgress.total > 0 ? Math.round((analyzeProgress.current / analyzeProgress.total) * 100) : 0}%)
              </span>
              {onCancelAnalysis && (
                <button
                  onClick={onCancelAnalysis}
                  className="px-2.5 py-1 rounded-lg text-xs font-semibold bg-rose-600/90 hover:bg-rose-500 text-white transition flex items-center gap-1 cursor-pointer shadow"
                  title="Stop analysis process"
                >
                  <Square className="w-3 h-3 fill-current" />
                  Stop
                </button>
              )}
            </div>
          </div>
          <div className="w-full bg-slate-900 rounded-full h-2.5 overflow-hidden">
            <div
              className="bg-gradient-to-r from-cyan-500 to-indigo-500 h-2.5 rounded-full transition-all duration-300"
              style={{
                width: `${analyzeProgress.total > 0 ? (analyzeProgress.current / analyzeProgress.total) * 100 : 0}%`,
              }}
            />
          </div>
        </div>
      )}

      {/* Top Hero Section */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-br from-slate-900 via-slate-900/90 to-indigo-950/40 border border-slate-800 p-8 shadow-2xl">
        <div className="absolute top-0 right-0 -mr-20 -mt-20 w-96 h-96 rounded-full bg-indigo-600/10 blur-3xl pointer-events-none" />
        <div className="relative z-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
          <div className="max-w-2xl space-y-3">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-xs font-semibold text-indigo-300">
              <Cpu className="w-3.5 h-3.5" />
              RTX 3050 & CPU Optimized Local Pipeline
            </div>
            <h1 className="text-3xl font-extrabold tracking-tight text-white">
              Intelligent Local File Organizer
            </h1>
            <p className="text-sm text-slate-300 leading-relaxed">
              Zorter uses locally running vision models to inspect images, recognize aesthetic styles and scenes, generate deterministic filenames, and organize messy wallpaper folders without cloud dependencies.
            </p>
          </div>

          <div className="flex flex-col sm:flex-row items-center gap-3 shrink-0">
            <button
              onClick={onSelectDirectory}
              disabled={isScanning || isAnalyzing}
              className="px-6 py-3.5 rounded-xl font-semibold text-sm bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white shadow-xl shadow-indigo-600/25 transition-all flex items-center gap-2 group cursor-pointer"
            >
              <FolderOpen className="w-4 h-4 group-hover:scale-110 transition-transform" />
              Scan a Directory
            </button>
            <button
              onClick={() => onNavigateTab('organize')}
              className="px-5 py-3.5 rounded-xl font-medium text-sm bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition flex items-center gap-2 cursor-pointer"
            >
              Organize Files
              <ArrowRight className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* Metrics Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Total Files */}
        <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800 hover:border-slate-700 transition">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-medium uppercase tracking-wider">Indexed Files</span>
            <div className="p-2 rounded-xl bg-slate-800 text-slate-200">
              <ImageIcon className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-bold text-white tracking-tight">
            {stats?.total_files ?? 0}
          </div>
          <p className="text-xs text-slate-400 mt-1">Images indexed with BLAKE3</p>
        </div>

        {/* AI Analyzed */}
        <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800 hover:border-slate-700 transition">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-medium uppercase tracking-wider">AI Analyzed</span>
            <div className="p-2 rounded-xl bg-indigo-500/20 text-indigo-300">
              <Brain className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-bold text-white tracking-tight">
            {stats?.total_analyzed ?? 0}
          </div>
          <p className="text-xs text-slate-400 mt-1">Structured visual descriptions</p>
        </div>

        {/* Semantic Embeddings */}
        <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800 hover:border-slate-700 transition">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-medium uppercase tracking-wider">Vector Embeddings</span>
            <div className="p-2 rounded-xl bg-cyan-500/20 text-cyan-300">
              <Search className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-bold text-white tracking-tight">
            {stats?.total_embeddings ?? 0}
          </div>
          <p className="text-xs text-slate-400 mt-1">Local semantic search index</p>
        </div>

        {/* Duplicates */}
        <div className="p-5 rounded-2xl bg-slate-900/60 border border-slate-800 hover:border-slate-700 transition">
          <div className="flex items-center justify-between text-slate-400 mb-3">
            <span className="text-xs font-medium uppercase tracking-wider">Duplicates</span>
            <div className="p-2 rounded-xl bg-amber-500/20 text-amber-300">
              <Copy className="w-4 h-4" />
            </div>
          </div>
          <div className="text-2xl font-bold text-white tracking-tight">
            {stats?.total_duplicates ?? 0}
          </div>
          <p className="text-xs text-slate-400 mt-1">
            {formatBytes(stats?.wasted_bytes ?? 0)} duplicate space
          </p>
        </div>
      </div>

      {/* Two Column Section */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column: Model Status & Config (1 col) */}
        <div className="space-y-6">
          <div className="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <HardDrive className="w-4 h-4 text-indigo-400" />
                Active Local Models
              </h3>
              <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[11px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                <CheckCircle2 className="w-3 h-3" />
                Ready
              </span>
            </div>

            <div className="space-y-3 text-xs">
              <div className="p-3 rounded-xl bg-slate-950/80 border border-slate-800/80">
                <div className="text-slate-400 font-medium">Vision Intelligence:</div>
                <div className="text-slate-200 font-mono font-semibold mt-0.5">
                  {settings?.preferred_vision_model}
                </div>
                <div className="text-[11px] text-slate-400 mt-1">
                  Extracts style, category, lighting, colors & orientation
                </div>
              </div>

              <div className="p-3 rounded-xl bg-slate-950/80 border border-slate-800/80">
                <div className="text-slate-400 font-medium">Semantic Embeddings:</div>
                <div className="text-slate-200 font-mono font-semibold mt-0.5">
                  {settings?.preferred_embedding_model}
                </div>
                <div className="text-[11px] text-slate-400 mt-1">
                  Enables natural-language similarity vector search
                </div>
              </div>

              <div className="p-3 rounded-xl bg-slate-950/80 border border-slate-800/80">
                <div className="text-slate-400 font-medium">Naming Convention:</div>
                <div className="text-indigo-300 font-mono font-semibold mt-0.5">
                  {settings?.default_naming_convention}
                </div>
              </div>
            </div>
          </div>

          {/* Safety & Persistence info card */}
          <div className="p-5 rounded-2xl bg-slate-900/40 border border-slate-800/80 text-xs text-slate-400 space-y-2">
            <div className="font-semibold text-slate-300 flex items-center gap-2">
              <Layers className="w-4 h-4 text-cyan-400" />
              Safety & Rollback Journal
            </div>
            <p>
              All file renames and moves are logged in an append-only JSONL journal with atomic replacements and .bak backups. Operations can be rolled back at any time with a single click.
            </p>
          </div>
        </div>

        {/* Right Column: Recent Activity & Files (2 cols) */}
        <div className="lg:col-span-2 space-y-6">
          {/* Recent Operations Activity */}
          <div className="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <Clock className="w-4 h-4 text-cyan-400" />
                Recent Operations History
              </h3>
              {recentHistory.length > 0 && (
                <button
                  onClick={onUndoLast}
                  className="text-xs text-amber-400 hover:text-amber-300 font-medium flex items-center gap-1"
                >
                  <RotateCcw className="w-3.5 h-3.5" />
                  Undo Last Batch
                </button>
              )}
            </div>

            {recentHistory.length === 0 ? (
              <div className="text-center py-8 text-xs text-slate-400 border border-dashed border-slate-800 rounded-xl">
                No organization operations executed yet. Scan a directory and approve an organization plan to begin.
              </div>
            ) : (
              <div className="space-y-2">
                {recentHistory.slice(-4).reverse().map((op) => (
                  <div
                    key={op.id}
                    className="p-3 rounded-xl bg-slate-950/60 border border-slate-800/70 flex items-center justify-between text-xs"
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-2 h-2 rounded-full ${
                          op.status === 'completed'
                            ? 'bg-emerald-400'
                            : op.status === 'rolled_back'
                            ? 'bg-amber-400'
                            : 'bg-red-400'
                        }`}
                      />
                      <div>
                        <div className="font-medium text-slate-200">
                          {op.operation_type === 'batch_organize'
                            ? 'Batch Organized Files'
                            : op.operation_type.startsWith('rollback:')
                            ? 'Rollback Executed'
                            : op.operation_type}
                        </div>
                        <div className="text-[11px] text-slate-400">
                          {op.actions.length} files affected • {new Date(op.timestamp).toLocaleTimeString()}
                        </div>
                      </div>
                    </div>
                    <span
                      className={`text-[10px] uppercase font-bold px-2 py-0.5 rounded-full ${
                        op.status === 'completed'
                          ? 'bg-emerald-500/10 text-emerald-400'
                          : 'bg-amber-500/10 text-amber-400'
                      }`}
                    >
                      {op.status}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Quick Preview of Indexed Files */}
          <div className="p-6 rounded-2xl bg-slate-900/60 border border-slate-800 space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-white flex items-center gap-2">
                <ImageIcon className="w-4 h-4 text-indigo-400" />
                Recently Indexed Files
              </h3>
              <button
                onClick={() => onNavigateTab('explore')}
                className="text-xs text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1"
              >
                View all in Explore
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>

            {recentFiles.length === 0 ? (
              <div className="text-center py-8 text-xs text-slate-400 border border-dashed border-slate-800 rounded-xl">
                No files indexed yet. Click "Scan a Directory" above to start indexing images.
              </div>
            ) : (
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                {recentFiles.slice(0, 8).map((file) => (
                  <div
                    key={file.id}
                    className="group relative rounded-xl overflow-hidden bg-slate-950 border border-slate-800 aspect-square flex flex-col"
                  >
                    {file.thumbnail_url ? (
                      <img
                        src={file.thumbnail_url}
                        alt={file.current_name}
                        className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center text-slate-700 bg-slate-950">
                        <ImageIcon className="w-8 h-8" />
                      </div>
                    )}
                    <div className="absolute inset-x-0 bottom-0 p-2 bg-gradient-to-t from-black/80 via-black/40 to-transparent">
                      <p className="text-[11px] font-medium text-slate-200 truncate">
                        {file.current_name}
                      </p>
                      <p className="text-[9px] text-slate-400 font-mono">
                        {(file.size / 1024).toFixed(0)} KB
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
