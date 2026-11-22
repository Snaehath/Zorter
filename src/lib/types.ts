// Mirrors the Rust payloads in src-tauri/src/commands and core/events.rs.

export interface ModelStatus {
  kind: 'Vision' | 'Embedding';
  name: string;
  installed: boolean;
  size_bytes: number;
}

export interface Overview {
  version: string;
  endpoint: string;
  online: boolean;
  models: ModelStatus[];
  files: number;
  analyzed: number;
  searchable: number;
  storage_path: string;
  storage_bytes: number;
  last_scan: string | null;
  last_analysis: string | null;
  operations_completed: number;
  operations_failed: number;
  operations_undone: number;
  undoable: { timestamp: string; files: number } | null;
}

export interface ScanReport {
  directory: string;
  files_discovered: number;
  images: number;
  documents: number;
  other: number;
  already_indexed: number;
  new_files: number;
  needs_analysis: number;
  duplicates: number;
  wasted_bytes: number;
  elapsed_ms: number;
}

export interface Progress {
  current: number;
  total: number;
}

export interface FileAnalysis {
  file_id: string;
  description: string;
  subject: string[];
  category: string;
  subcategory: string;
  tags: string[];
  visual_style: string;
  environment?: string | null;
  lighting?: string | null;
  colors: string[];
  orientation?: string | null;
  suggested_filename: string;
  model: string;
  analyzed_at: string;
}

export interface AnalyzeStarted {
  total: number;
  already_analyzed: number;
  model: string;
  search_model: string | null;
}

export interface AnalyzeFileEvent {
  index: number;
  total: number;
  file_id: string;
  name: string;
  path: string;
  hash: string;
  analysis: FileAnalysis | null;
  proposed_name: string | null;
  error: string | null;
}

export interface AnalyzeSummary {
  total: number;
  analyzed: number;
  failed: number;
  cancelled: boolean;
  search_indexed: number;
  elapsed_ms: number;
}

export interface PlannedChange {
  file_id: string;
  from: string;
  to: string;
  original_name: string;
  proposed_name: string;
  folder: string;
  uncertain: boolean;
  description: string;
  hash: string;
}

export interface PlanOptions {
  layout?: 'by_category' | 'category' | 'category_and_style' | 'in_place' | 'flat_rename';
  naming?: 'subject_scene' | 'kebab-case' | 'title_case' | 'detailed' | 'ai_suggested' | 'original';
  move_unanalyzed?: boolean;
}

export interface PlanReport {
  root: string;
  root_name: string;
  total_files: number;
  analyzed: number;
  ready: number;
  review: number;
  unchanged: number;
  skipped_unanalyzed: number;
  folders: { name: string; count: number }[];
  changes: PlannedChange[];
}

export interface SearchHit {
  rank: number;
  score: number;
  name: string;
  path: string;
  folder: string;
  hash: string;
  description: string;
  category: string;
}

export interface SearchResponse {
  mode: 'semantic' | 'keyword';
  reason: string | null;
  model: string;
  total: number;
  hits: SearchHit[];
}
