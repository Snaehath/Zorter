import React, { useState } from 'react';
import {
  Brain,
  FolderTree,
  CheckSquare,
  Square,
  ArrowRight,
  FolderOpen,
  Sparkles,
  ShieldCheck,
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
  Zap,
} from 'lucide-react';
import type {
  AnalyzeProgressEvent,
  FileAnalysis,
  IndexedFile,
  OrganizationPlan,
} from '../types';
import { api } from '../services/api';

interface OrganizeScreenProps {
  files: IndexedFile[];
  analyses: FileAnalysis[];
  onSelectDirectory: () => void;
  onRefreshData: () => void;
  onUndoLast: () => void;
  isUndoAvailable: boolean;
  isAnalyzing: boolean;
  analyzeProgress: AnalyzeProgressEvent | null;
  onRunAnalysis: () => void;
  onRunFastHeuristic: () => void;
  onCancelAnalysis: () => void;
}

export const OrganizeScreen: React.FC<OrganizeScreenProps> = ({
  files,
  analyses,
  onSelectDirectory,
  onRefreshData,
  onUndoLast,
  isUndoAvailable,
  isAnalyzing,
  analyzeProgress,
  onRunAnalysis,
  onRunFastHeuristic,
  onCancelAnalysis,
}) => {
  const [namingConvention, setNamingConvention] = useState<string>('subject_scene');
  const [organizationBehavior, setOrganizationBehavior] = useState<string>('by_category');
  const [plan, setPlan] = useState<OrganizationPlan | null>(null);
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [isPlanning, setIsPlanning] = useState(false);
  const [isExecuting, setIsExecuting] = useState(false);
  const [executionMessage, setExecutionMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Analysis lookup map
  const analysisMap = new Map<string, FileAnalysis>();
  analyses.forEach((a) => analysisMap.set(a.file_id, a));

  const analyzedCount = files.filter((f) => analysisMap.has(f.id)).length;
  const unanalyzedCount = files.length - analyzedCount;

  const handleGeneratePlan = async () => {
    setIsPlanning(true);
    setErrorMessage(null);
    setExecutionMessage(null);
    try {
      const generatedPlan = await api.planOrganization(
        undefined,
        namingConvention,
        organizationBehavior
      );
      setPlan(generatedPlan);
      // Default all items to selected
      const allIds = new Set(generatedPlan.items.map((i) => i.file_id));
      setSelectedItems(allIds);
    } catch (e) {
      setErrorMessage(`Failed to create organization plan: ${e}`);
    } finally {
      setIsPlanning(false);
    }
  };

  const handleAnalyzeAndPlan = async () => {
    setErrorMessage(null);
    setExecutionMessage(null);
    // 1. Run analysis
    onRunAnalysis();
  };

  const toggleItem = (fileId: string) => {
    const updated = new Set(selectedItems);
    if (updated.has(fileId)) {
      updated.delete(fileId);
    } else {
      updated.add(fileId);
    }
    setSelectedItems(updated);
  };

  const selectAll = () => {
    if (!plan) return;
    setSelectedItems(new Set(plan.items.map((i) => i.file_id)));
  };

  const deselectAll = () => {
    setSelectedItems(new Set());
  };

  const handleExecute = async () => {
    if (!plan) return;
    const itemsToExecute = plan.items.filter((i) => selectedItems.has(i.file_id));
    if (itemsToExecute.length === 0) {
      setErrorMessage('Please select at least one file to organize.');
      return;
    }

    setIsExecuting(true);
    setErrorMessage(null);
    setExecutionMessage(null);

    try {
      const actions = itemsToExecute.map((item) => ({
        file_id: item.file_id,
        source_path: item.original_path,
        destination_path: item.proposed_destination_path,
        action_type: item.action_type,
      }));

      const result = await api.executeOrganization(actions);
      setExecutionMessage(
        `Successfully organized ${result.actions.length} files! Operation journal logged.`
      );
      setPlan(null);
      setSelectedItems(new Set());
      onRefreshData();
    } catch (e) {
      setErrorMessage(`Execution error: ${e}`);
    } finally {
      setIsExecuting(false);
    }
  };

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-16">
      {/* Active Analysis Progress Bar with Stop Control */}
      {isAnalyzing && (
        <div className="p-4 rounded-2xl bg-cyan-950/70 border border-cyan-500/40 shadow-xl shadow-cyan-950/50 flex flex-col gap-3">
          <div className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-2 text-cyan-300 font-medium truncate max-w-[70%]">
              <Brain className="w-4 h-4 text-cyan-400 shrink-0 animate-pulse" />
              <span className="truncate">
                {analyzeProgress?.status || 'Analyzing images'}:{' '}
                <strong className="text-white">{analyzeProgress?.file_name || 'Processing...'}</strong>
              </span>
            </div>
            <div className="flex items-center gap-3 shrink-0">
              <span className="font-mono text-cyan-200">
                {analyzeProgress ? `${analyzeProgress.current} / ${analyzeProgress.total}` : 'In progress...'}
                {analyzeProgress && analyzeProgress.total > 0
                  ? ` (${Math.round((analyzeProgress.current / analyzeProgress.total) * 100)}%)`
                  : ''}
              </span>
              <button
                onClick={onCancelAnalysis}
                className="px-3 py-1.5 rounded-lg text-xs font-bold bg-rose-600 hover:bg-rose-500 text-white transition flex items-center gap-1.5 cursor-pointer shadow-lg shadow-rose-900/40"
                title="Stop analysis process"
              >
                <Square className="w-3.5 h-3.5 fill-current" />
                Stop Process
              </button>
            </div>
          </div>
          <div className="w-full bg-slate-900 rounded-full h-2.5 overflow-hidden">
            <div
              className="bg-gradient-to-r from-cyan-500 via-indigo-500 to-emerald-400 h-2.5 rounded-full transition-all duration-300"
              style={{
                width: `${
                  analyzeProgress && analyzeProgress.total > 0
                    ? (analyzeProgress.current / analyzeProgress.total) * 100
                    : 15
                }%`,
              }}
            />
          </div>
        </div>
      )}

      {/* Top Controls Bar */}
      <div className="p-6 rounded-2xl bg-slate-900/80 border border-slate-800 shadow-xl space-y-6">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
          <div>
            <h2 className="text-xl font-bold text-white flex items-center gap-2.5">
              <FolderTree className="w-5 h-5 text-indigo-400" />
              Analyze & Organize
            </h2>
            <p className="text-xs text-slate-400 mt-1">
              {files.length} files indexed • <span className="text-emerald-400 font-semibold">{analyzedCount} analyzed</span>
              {unanalyzedCount > 0 && (
                <span className="text-amber-400 ml-1 font-semibold">({unanalyzedCount} waiting for AI analysis)</span>
              )}
            </p>
          </div>

          <div className="flex flex-wrap items-center gap-2.5">
            <button
              onClick={onSelectDirectory}
              className="px-4 py-2 rounded-xl text-xs font-semibold bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 transition flex items-center gap-2 cursor-pointer"
            >
              <FolderOpen className="w-3.5 h-3.5" />
              Change Folder
            </button>

            {/* Instant Fast Auto-Name (0s / Heuristic / Embedded Prompts) */}
            <button
              onClick={onRunFastHeuristic}
              disabled={isAnalyzing || files.length === 0}
              title="Reads embedded PNG generation prompts (Flux, ComfyUI, Automatic1111) and smart heuristics in 0s without waiting on AI"
              className={`px-4 py-2.5 rounded-xl text-xs font-bold transition flex items-center gap-2 cursor-pointer ${
                isAnalyzing && analyzeProgress?.status.includes('Instant')
                  ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 animate-pulse'
                  : 'bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white shadow-lg shadow-emerald-600/20'
              }`}
            >
              <Zap className="w-4 h-4 text-emerald-200" />
              {isAnalyzing && analyzeProgress?.status.includes('Instant')
                ? 'Extracting Prompts...'
                : 'Instant Auto-Name (0s)'}
            </button>

            {/* Deep AI Vision Analysis Button */}
            <button
              onClick={handleAnalyzeAndPlan}
              disabled={isAnalyzing || files.length === 0}
              title="Full visual image inspection with local Ollama vision model (~2-3s per image)"
              className={`px-4 py-2.5 rounded-xl text-xs font-bold transition flex items-center gap-2 cursor-pointer ${
                isAnalyzing && !analyzeProgress?.status.includes('Instant')
                  ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 animate-pulse'
                  : 'bg-gradient-to-r from-cyan-600 to-indigo-600 hover:from-cyan-500 hover:to-indigo-500 text-white shadow-lg shadow-indigo-600/25'
              }`}
            >
              <Brain className="w-4 h-4 text-cyan-300" />
              {isAnalyzing && !analyzeProgress?.status.includes('Instant')
                ? 'AI Vision in Progress...'
                : unanalyzedCount > 0
                ? `Deep AI Vision (${unanalyzedCount})`
                : 'Re-Analyze with AI'}
            </button>

            <button
              onClick={handleGeneratePlan}
              disabled={isPlanning || files.length === 0}
              className="px-4 py-2.5 rounded-xl text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white shadow-lg shadow-indigo-600/20 transition flex items-center gap-2 cursor-pointer"
            >
              <Sparkles className="w-3.5 h-3.5" />
              {isPlanning ? 'Generating Plan...' : 'Generate Plan'}
            </button>

            {isUndoAvailable && (
              <button
                onClick={onUndoLast}
                className="px-3.5 py-2 rounded-xl text-xs font-medium bg-amber-500/10 text-amber-300 border border-amber-500/30 hover:bg-amber-500/20 transition flex items-center gap-1.5 cursor-pointer"
              >
                <RotateCcw className="w-3.5 h-3.5" />
                Undo
              </button>
            )}
          </div>
        </div>

        {/* Naming Style & Folder Hierarchy Configuration */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-4 border-t border-slate-800/80 text-xs">
          <div className="space-y-1.5">
            <label className="text-slate-400 font-medium">Naming Style (How files are renamed)</label>
            <select
              value={namingConvention}
              onChange={(e) => setNamingConvention(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="subject_scene">Subject & Scene (e.g. egypt_pyramids, waterfalls, forest)</option>
              <option value="ai_suggested">AI Suggested (e.g. egyptian_pyramids, autumn_forest)</option>
              <option value="kebab-case">Kebab-case (e.g. egypt-pyramids, waterfalls, forest)</option>
              <option value="title_case">Title_Case (e.g. Egypt_Pyramids, Waterfalls, Forest)</option>
            </select>
          </div>

          <div className="space-y-1.5">
            <label className="text-slate-400 font-medium">Folder Organization Hierarchy</label>
            <select
              value={organizationBehavior}
              onChange={(e) => setOrganizationBehavior(e.target.value)}
              className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-slate-200 focus:outline-none focus:border-indigo-500"
            >
              <option value="by_category">By Category & Subcategory (Wallpapers/Nature/Waterfalls/)</option>
              <option value="category_and_style">By Category & Visual Style (Wallpapers/Digital Art/)</option>
              <option value="flat_rename">Flat Rename in Current Directory (Keep in Same Folder)</option>
            </select>
          </div>
        </div>
      </div>

      {/* Prominent Notice if Unanalyzed Files Exist */}
      {unanalyzedCount > 0 && (
        <div className="p-4 rounded-2xl bg-amber-950/40 border border-amber-500/30 text-amber-200 text-xs flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-lg">
          <div className="flex items-center gap-2.5">
            <AlertTriangle className="w-5 h-5 text-amber-400 shrink-0" />
            <div>
              <span className="font-semibold text-white">
                {unanalyzedCount} file(s) have not been analyzed yet.
              </span>
              <p className="text-[11px] text-amber-300/80 mt-0.5">
                Without analysis, files keep their original name. Choose <strong>Instant (0s)</strong> to immediately extract embedded prompts/metadata, or <strong>Deep AI</strong> to let Ollama inspect each image visually.
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <button
              onClick={onRunFastHeuristic}
              disabled={isAnalyzing}
              className="px-3.5 py-2 rounded-xl text-xs font-bold bg-emerald-500 hover:bg-emerald-400 text-slate-950 transition cursor-pointer flex items-center gap-1.5 shadow"
            >
              <Zap className="w-3.5 h-3.5" />
              Instant (0s)
            </button>
            <button
              onClick={handleAnalyzeAndPlan}
              disabled={isAnalyzing}
              className="px-3.5 py-2 rounded-xl text-xs font-bold bg-amber-500 hover:bg-amber-400 text-slate-950 transition cursor-pointer flex items-center gap-1.5 shadow"
            >
              <Brain className="w-3.5 h-3.5" />
              {isAnalyzing ? 'Analyzing...' : 'Deep AI Vision'}
            </button>
          </div>
        </div>
      )}

      {/* Status Notifications */}
      {executionMessage && (
        <div className="p-4 rounded-xl bg-emerald-950/40 border border-emerald-500/30 text-emerald-300 text-xs flex items-center gap-2">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{executionMessage}</span>
        </div>
      )}

      {errorMessage && (
        <div className="p-4 rounded-xl bg-red-950/40 border border-red-500/30 text-red-300 text-xs flex items-center gap-2">
          <AlertTriangle className="w-4 h-4 text-red-400 shrink-0" />
          <span>{errorMessage}</span>
        </div>
      )}

      {/* Plan Preview Section */}
      {plan ? (
        <div className="space-y-4">
          {/* Action Ribbon */}
          <div className="p-4 rounded-2xl bg-slate-900 border border-slate-800 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
            <div className="flex items-center gap-3 text-xs">
              <span className="font-semibold text-white">
                Proposed Plan: {plan.items.length} items
              </span>
              <span className="text-slate-500">•</span>
              <span className="text-indigo-400 font-medium">
                {selectedItems.size} selected for organization
              </span>
              <div className="flex items-center gap-1.5 ml-2">
                <button
                  onClick={selectAll}
                  className="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-[11px]"
                >
                  Select All
                </button>
                <button
                  onClick={deselectAll}
                  className="px-2.5 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-[11px]"
                >
                  Deselect All
                </button>
              </div>
            </div>

            <button
              onClick={handleExecute}
              disabled={isExecuting || selectedItems.size === 0}
              className="px-5 py-2.5 rounded-xl font-semibold text-xs bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-600/20 transition flex items-center gap-2 cursor-pointer"
            >
              <ShieldCheck className="w-4 h-4" />
              {isExecuting
                ? 'Safely Executing...'
                : `Approve & Organize (${selectedItems.size} Files)`}
            </button>
          </div>

          {/* Plan Items List */}
          <div className="grid grid-cols-1 gap-3">
            {plan.items.map((item) => {
              const analysis = analysisMap.get(item.file_id);
              const isSelected = selectedItems.has(item.file_id);

              return (
                <div
                  key={item.file_id}
                  onClick={() => toggleItem(item.file_id)}
                  className={`p-4 rounded-2xl border transition-all cursor-pointer flex flex-col sm:flex-row items-start sm:items-center gap-4 ${
                    isSelected
                      ? 'bg-slate-900/90 border-indigo-500/40 shadow-lg shadow-indigo-950/20'
                      : 'bg-slate-950/60 border-slate-800 opacity-60'
                  }`}
                >
                  {/* Selection Checkbox */}
                  <div className="shrink-0 text-indigo-400">
                    {isSelected ? (
                      <CheckSquare className="w-5 h-5 text-indigo-400" />
                    ) : (
                      <Square className="w-5 h-5 text-slate-600" />
                    )}
                  </div>

                  {/* Thumbnail */}
                  <div className="w-16 h-16 rounded-xl bg-slate-950 border border-slate-800 overflow-hidden shrink-0">
                    {item.thumbnail_url ? (
                      <img
                        src={item.thumbnail_url}
                        alt={item.original_name}
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center text-slate-700 text-xs">
                        IMG
                      </div>
                    )}
                  </div>

                  {/* Comparison Details */}
                  <div className="flex-1 min-w-0 space-y-1.5">
                    {/* Filename Transformation */}
                    <div className="flex flex-wrap items-center gap-2 text-xs">
                      <span className="font-mono text-slate-400 line-through truncate max-w-xs" title={item.original_name}>
                        {item.original_name}
                      </span>
                      <ArrowRight className="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                      <span className="font-mono font-bold text-emerald-300 truncate max-w-sm" title={item.proposed_name}>
                        {item.proposed_name}
                      </span>
                      {!item.is_analyzed && (
                        <span className="text-[10px] font-bold px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/30">
                          Needs AI Analysis
                        </span>
                      )}
                    </div>

                    {/* Destination Path */}
                    <div className="text-[11px] text-slate-400 flex items-center gap-1 font-mono truncate">
                      <span className="text-slate-500">Destination:</span>
                      <span className="text-slate-300 truncate">{item.proposed_destination_path}</span>
                    </div>

                    {/* Analysis Badges */}
                    {analysis ? (
                      <div className="flex flex-wrap items-center gap-1.5 pt-1">
                        <span className="text-[10px] font-semibold px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                          {item.category} / {item.subcategory}
                        </span>
                        {analysis.subject.slice(0, 2).map((s, idx) => (
                          <span
                            key={idx}
                            className="text-[10px] px-2 py-0.5 rounded-full bg-cyan-950 text-cyan-300 border border-cyan-800"
                          >
                            {s}
                          </span>
                        ))}
                        {analysis.environment && (
                          <span className="text-[10px] px-2 py-0.5 rounded-full bg-slate-800 text-slate-300">
                            {analysis.environment}
                          </span>
                        )}
                      </div>
                    ) : (
                      <div className="text-[11px] text-slate-500 italic">
                        Not analyzed yet. Click "Run AI Analysis" to extract subject, scene, and location.
                      </div>
                    )}
                  </div>

                  {/* Action Badge */}
                  <div className="shrink-0 text-right">
                    <span className="text-[10px] font-bold uppercase px-2.5 py-1 rounded-full bg-slate-800 text-slate-300 border border-slate-700">
                      {item.action_type.replace('_', ' ')}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ) : (
        /* Empty State / Initial View */
        <div className="p-12 rounded-2xl bg-slate-900/40 border border-dashed border-slate-800 text-center space-y-4">
          <div className="w-12 h-12 rounded-2xl bg-indigo-500/10 text-indigo-400 flex items-center justify-center mx-auto">
            <Sparkles className="w-6 h-6" />
          </div>
          <div className="max-w-md mx-auto space-y-1">
            <h3 className="text-sm font-semibold text-white">
              Ready to Rename & Organize Images
            </h3>
            <p className="text-xs text-slate-400">
              Click <span className="text-cyan-400 font-semibold">"Run AI Analysis"</span> so local AI can understand the content of your images and generate names like <span className="text-emerald-300 font-medium">egypt_pyramids</span>, <span className="text-emerald-300 font-medium">waterfalls</span>, and <span className="text-emerald-300 font-medium">forest</span>.
            </p>
          </div>
          <div className="flex items-center justify-center gap-3">
            <button
              onClick={handleAnalyzeAndPlan}
              disabled={isAnalyzing || files.length === 0}
              className="px-5 py-2.5 rounded-xl font-bold text-xs bg-gradient-to-r from-cyan-600 to-indigo-600 hover:from-cyan-500 hover:to-indigo-500 text-white shadow-lg shadow-indigo-600/20 transition inline-flex items-center gap-2 cursor-pointer"
            >
              <Zap className="w-3.5 h-3.5 text-cyan-300" />
              {isAnalyzing ? 'Analyzing in Background...' : 'Run AI Analysis & Generate Plan'}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
