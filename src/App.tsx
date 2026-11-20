import { useState, useEffect } from 'react';
import { Header } from './components/Header';
import { DashboardScreen } from './components/DashboardScreen';
import { OrganizeScreen } from './components/OrganizeScreen';
import { ExploreScreen } from './components/ExploreScreen';
import { SettingsModal } from './components/SettingsModal';
import type {
  AppSettings,
  AppStats,
  FileAnalysis,
  IndexedFile,
  OperationJournalEntry,
  ScanProgressEvent,
  AnalyzeProgressEvent,
} from './types';
import { api } from './services/api';

export function App() {
  const [currentTab, setCurrentTab] = useState<'dashboard' | 'organize' | 'explore'>('dashboard');
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [files, setFiles] = useState<IndexedFile[]>([]);
  const [analyses, setAnalyses] = useState<FileAnalysis[]>([]);
  const [stats, setStats] = useState<AppStats | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [history, setHistory] = useState<OperationJournalEntry[]>([]);

  // Async Tasks State
  const [isScanning, setIsScanning] = useState(false);
  const [scanProgress, setScanProgress] = useState<ScanProgressEvent | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analyzeProgress, setAnalyzeProgress] = useState<AnalyzeProgressEvent | null>(null);

  const loadData = async () => {
    try {
      const [fetchedFiles, fetchedStats, fetchedSettings, fetchedHistory] = await Promise.all([
        api.getIndexedFiles().catch(() => []),
        api.getAppStats().catch(() => null),
        api.getSettings().catch(() => null),
        api.getOperationHistory().catch(() => []),
      ]);

      setFiles(fetchedFiles);
      setStats(fetchedStats);
      setSettings(fetchedSettings);
      setHistory(fetchedHistory);

      // Load analyses for each file
      if (fetchedFiles.length > 0) {
        const loadedAnalyses: FileAnalysis[] = [];
        for (const file of fetchedFiles) {
          const a = await api.getFileAnalysis(file.id).catch(() => null);
          if (a) loadedAnalyses.push(a);
        }
        setAnalyses(loadedAnalyses);
      }
    } catch (e) {
      console.error('Failed to load application data', e);
    }
  };

  useEffect(() => {
    loadData();

    // Setup event listeners
    let unlistenScanProg: (() => void) | undefined;
    let unlistenScanDone: (() => void) | undefined;
    let unlistenAnalyzeProg: (() => void) | undefined;
    let unlistenAnalyzeDone: (() => void) | undefined;

    api.onScanProgress((e) => {
      setIsScanning(true);
      setScanProgress(e);
    }).then((un) => {
      unlistenScanProg = un;
    });

    api.onScanCompleted(() => {
      setIsScanning(false);
      setScanProgress(null);
      loadData();
    }).then((un) => {
      unlistenScanDone = un;
    });

    api.onAnalyzeProgress((e) => {
      setIsAnalyzing(true);
      setAnalyzeProgress(e);
    }).then((un) => {
      unlistenAnalyzeProg = un;
    });

    api.onAnalyzeCompleted(() => {
      setIsAnalyzing(false);
      setAnalyzeProgress(null);
      loadData();
    }).then((un) => {
      unlistenAnalyzeDone = un;
    });

    return () => {
      if (unlistenScanProg) unlistenScanProg();
      if (unlistenScanDone) unlistenScanDone();
      if (unlistenAnalyzeProg) unlistenAnalyzeProg();
      if (unlistenAnalyzeDone) unlistenAnalyzeDone();
    };
  }, []);

  const handleSelectDirectory = async () => {
    try {
      const selected = await api.selectDirectory();
      if (selected) {
        setIsScanning(true);
        await api.scanDirectory(selected);
        await loadData();
      }
    } catch (e) {
      console.error('Directory selection/scan error', e);
    } finally {
      setIsScanning(false);
      setScanProgress(null);
    }
  };

  const handleRunAnalysis = async () => {
    setIsAnalyzing(true);
    try {
      await api.analyzeFiles();
      await loadData();
    } catch (e) {
      console.error('Analysis error', e);
    } finally {
      setIsAnalyzing(false);
      setAnalyzeProgress(null);
    }
  };

  const handleRunFastHeuristic = async () => {
    setIsAnalyzing(true);
    try {
      await api.fastHeuristicAnalyze();
      await loadData();
    } catch (e) {
      console.error('Fast heuristic analysis error', e);
    } finally {
      setIsAnalyzing(false);
      setAnalyzeProgress(null);
    }
  };

  const handleUndoLast = async () => {
    try {
      await api.undoLastOperation();
      await loadData();
    } catch (e) {
      alert(`Undo error: ${e}`);
    }
  };

  const hasUndoAvailable = history.some(
    (op) => op.status === 'completed' && !op.operation_type.startsWith('rollback:')
  );

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-indigo-500 selection:text-white">
      {/* Top Header */}
      <Header
        currentTab={currentTab}
        onSelectTab={setCurrentTab}
        onOpenSettings={() => setIsSettingsOpen(true)}
        onUndoLast={handleUndoLast}
        isUndoAvailable={hasUndoAvailable}
        visionModel={settings?.preferred_vision_model || 'qwen3.5:4b'}
        embeddingModel={settings?.preferred_embedding_model || 'embeddinggemma-2:270m'}
        isProcessing={isScanning || isAnalyzing}
      />

      {/* Main View Area */}
      <main className="flex-1 p-6 md:p-8 max-w-[1600px] w-full mx-auto">
        {currentTab === 'dashboard' && (
          <DashboardScreen
            stats={stats}
            settings={settings}
            recentFiles={files}
            recentHistory={history}
            onSelectDirectory={handleSelectDirectory}
            onNavigateTab={setCurrentTab}
            onUndoLast={handleUndoLast}
            isScanning={isScanning}
            scanProgress={scanProgress}
            isAnalyzing={isAnalyzing}
            analyzeProgress={analyzeProgress}
          />
        )}

        {currentTab === 'organize' && (
          <OrganizeScreen
            files={files}
            analyses={analyses}
            onSelectDirectory={handleSelectDirectory}
            onRefreshData={loadData}
            onUndoLast={handleUndoLast}
            isUndoAvailable={hasUndoAvailable}
            isAnalyzing={isAnalyzing}
            onRunAnalysis={handleRunAnalysis}
            onRunFastHeuristic={handleRunFastHeuristic}
          />
        )}

        {currentTab === 'explore' && (
          <ExploreScreen files={files} analyses={analyses} />
        )}
      </main>

      {/* Settings Modal */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onSettingsSaved={loadData}
      />
    </div>
  );
}

export default App;
