import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type {
  AnalyzeProgressEvent,
  AppSettings,
  AppStats,
  DuplicateGroup,
  FileAnalysis,
  FileOperationAction,
  IndexedFile,
  ModelInfo,
  OperationJournalEntry,
  OrganizationPlan,
  ScanProgressEvent,
  SearchFilters,
  SearchResultItem,
} from '../types';

export const api = {
  // Directory & Scan
  selectDirectory: async (): Promise<string | null> => {
    return invoke<string | null>('select_directory');
  },

  scanDirectory: async (dirPath: string): Promise<IndexedFile[]> => {
    return invoke<IndexedFile[]>('scan_directory', { dirPath });
  },

  getIndexedFiles: async (): Promise<IndexedFile[]> => {
    return invoke<IndexedFile[]>('get_indexed_files');
  },

  // AI & Analysis
  listAvailableModels: async (): Promise<ModelInfo[]> => {
    return invoke<ModelInfo[]>('list_available_models');
  },

  getFileAnalysis: async (fileId: string): Promise<FileAnalysis | null> => {
    return invoke<FileAnalysis | null>('get_file_analysis', { fileId });
  },

  analyzeFiles: async (fileIds?: string[]): Promise<FileAnalysis[]> => {
    return invoke<FileAnalysis[]>('analyze_files', { fileIds });
  },

  fastHeuristicAnalyze: async (): Promise<FileAnalysis[]> => {
    return invoke<FileAnalysis[]>('fast_heuristic_analyze');
  },

  cancelAnalysis: async (): Promise<boolean> => {
    return invoke<boolean>('cancel_analysis');
  },

  // Organization
  planOrganization: async (
    targetDir?: string,
    namingConvention?: string,
    organizationBehavior?: string
  ): Promise<OrganizationPlan> => {
    return invoke<OrganizationPlan>('plan_organization', {
      targetDir,
      namingConvention,
      organizationBehavior,
    });
  },

  executeOrganization: async (
    actions: FileOperationAction[]
  ): Promise<OperationJournalEntry> => {
    return invoke<OperationJournalEntry>('execute_organization', { actions });
  },

  getDuplicateFiles: async (): Promise<DuplicateGroup[]> => {
    return invoke<DuplicateGroup[]>('get_duplicate_files');
  },

  // Search
  searchSemantic: async (
    query: string,
    filters?: SearchFilters,
    limit?: number
  ): Promise<SearchResultItem[]> => {
    return invoke<SearchResultItem[]>('search_semantic', {
      query,
      filters,
      limit,
    });
  },

  findSimilarImages: async (
    fileId: string,
    limit?: number
  ): Promise<SearchResultItem[]> => {
    return invoke<SearchResultItem[]>('find_similar_images', {
      fileId,
      limit,
    });
  },

  // Settings & Stats
  getSettings: async (): Promise<AppSettings> => {
    return invoke<AppSettings>('get_settings');
  },

  saveSettings: async (settings: AppSettings): Promise<void> => {
    return invoke<void>('save_settings', { settings });
  },

  getAppStats: async (): Promise<AppStats> => {
    return invoke<AppStats>('get_app_stats');
  },

  // Undo & History
  getOperationHistory: async (): Promise<OperationJournalEntry[]> => {
    return invoke<OperationJournalEntry[]>('get_operation_history');
  },

  undoLastOperation: async (): Promise<OperationJournalEntry> => {
    return invoke<OperationJournalEntry>('undo_last_operation');
  },

  // Event Listeners
  onScanProgress: (callback: (e: ScanProgressEvent) => void): Promise<UnlistenFn> => {
    return listen<ScanProgressEvent>('scan:progress', (event) => callback(event.payload));
  },

  onScanCompleted: (callback: () => void): Promise<UnlistenFn> => {
    return listen('scan:completed', () => callback());
  },

  onAnalyzeProgress: (callback: (e: AnalyzeProgressEvent) => void): Promise<UnlistenFn> => {
    return listen<AnalyzeProgressEvent>('analyze:progress', (event) => callback(event.payload));
  },

  onAnalyzeCompleted: (callback: () => void): Promise<UnlistenFn> => {
    return listen('analyze:completed', () => callback());
  },
};
