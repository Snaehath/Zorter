export interface IndexedFile {
  id: string;
  original_name: string;
  current_name: string;
  original_path: string;
  current_path: string;
  type: string;
  size: number;
  modified_ms: number;
  hash: string;
  thumbnail_url?: string | null;
  indexed: boolean;
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

export interface ModelInfo {
  name: string;
  size_bytes: number;
  modified_at: string;
  is_vision: boolean;
  is_embedding: boolean;
}

export interface AppSettings {
  preferred_vision_model: string;
  preferred_embedding_model: string;
  default_naming_convention: string;
  default_organization_behavior: string;
  thumbnail_size: number;
  inference_concurrency: number;
  selected_directories: string[];
  auto_monitoring: boolean;
  ollama_endpoint: string;
}

export interface AppStats {
  total_files: number;
  total_analyzed: number;
  total_embeddings: number;
  total_duplicates: number;
  wasted_bytes: number;
  monitored_directories: number;
}

export interface PlanItemPreview {
  file_id: string;
  original_name: string;
  original_path: string;
  proposed_name: string;
  proposed_destination_path: string;
  category: string;
  subcategory: string;
  action_type: string;
  thumbnail_url?: string | null;
  included: boolean;
  is_analyzed: boolean;
}

export interface OrganizationPlan {
  items: PlanItemPreview[];
  total_files: number;
  destination_root: string;
  unanalyzed_count: number;
}

export interface FileOperationAction {
  file_id: string;
  source_path: string;
  destination_path: string;
  action_type: string;
}

export interface OperationJournalEntry {
  id: string;
  timestamp: string;
  operation_type: string;
  status: string;
  actions: FileOperationAction[];
}

export interface DuplicateGroup {
  hash: string;
  total_size: number;
  wasted_bytes: number;
  files: IndexedFile[];
}

export interface SearchFilters {
  category?: string;
  subcategory?: string;
  tag?: string;
  visual_style?: string;
  orientation?: string;
  min_size?: number;
  max_size?: number;
}

export interface SearchResultItem {
  file: IndexedFile;
  analysis?: FileAnalysis | null;
  score: number;
}

export interface ScanProgressEvent {
  current: number;
  total: number;
  current_file: string;
}

export interface AnalyzeProgressEvent {
  current: number;
  total: number;
  file_name: string;
  status: string;
}
