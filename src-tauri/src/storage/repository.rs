use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::core::errors::{AppError, Result};
use crate::filesystem::operations::OperationJournalEntry;
use crate::storage::json_store::{append_jsonl, read_json_file, write_json_atomic};
use crate::storage::recovery::recover_or_load;

/// Vision models Zorter offers. The first one is the default; the rest are
/// alternatives the user can switch to from the Status screen.
pub const VISION_MODELS: [&str; 2] = ["minicpm-v4.6:latest", "ministral-3:3b"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub vision_model: String,
    pub preferred_embedding_model: String,
    pub default_naming_convention: String, // "clean_snake_case", "kebab-case", "title_case"
    pub default_organization_behavior: String, // "by_category", "flat_rename", "category_and_style"
    pub thumbnail_size: u32,
    pub inference_concurrency: usize,
    pub selected_directories: Vec<String>,
    pub auto_monitoring: bool,
    pub ollama_endpoint: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            vision_model: VISION_MODELS[0].to_string(),
            preferred_embedding_model: "embeddinggemma-2:270m".to_string(),
            default_naming_convention: "subject_scene".to_string(),
            default_organization_behavior: "by_category".to_string(),
            thumbnail_size: 256,
            inference_concurrency: 1, // RTX 3050 4GB VRAM safety
            selected_directories: Vec::new(),
            auto_monitoring: false,
            ollama_endpoint: "http://127.0.0.1:11434".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub id: String,
    pub original_name: String,
    pub current_name: String,
    pub original_path: String,
    pub current_path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: u64,
    pub modified_ms: u64,
    pub hash: String, // BLAKE3
    pub thumbnail_url: Option<String>,
    pub indexed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesIndex {
    pub version: u32,
    pub files: Vec<IndexedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub file_id: String,
    pub description: String,
    pub subject: Vec<String>,
    pub category: String,
    pub subcategory: String,
    pub tags: Vec<String>,
    pub visual_style: String,
    pub environment: Option<String>,
    pub lighting: Option<String>,
    pub colors: Vec<String>,
    pub orientation: Option<String>,
    pub suggested_filename: String,
    pub model: String,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisIndex {
    pub version: u32,
    pub analyses: Vec<FileAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEmbedding {
    pub file_id: String,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddingShard {
    pub version: u32,
    pub shard_id: usize,
    pub model: String,
    pub dimensions: usize,
    pub embeddings: Vec<FileEmbedding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredDirectory {
    pub path: String,
    pub added_at: String,
    pub total_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectoriesIndex {
    pub version: u32,
    pub directories: Vec<MonitoredDirectory>,
}

pub struct Repository {
    base_dir: PathBuf,
    settings: AppSettings,
    files: HashMap<String, IndexedFile>, // keyed by file_id
    analyses: HashMap<String, FileAnalysis>, // keyed by file_id
    embeddings: HashMap<String, Vec<f32>>, // keyed by file_id
    embedding_model: String,
    embedding_dims: usize,
    directories: Vec<MonitoredDirectory>,
    journal_path: PathBuf,
}

impl Repository {
    pub fn new(app_data_override: Option<PathBuf>) -> Result<Self> {
        let base_dir = if let Some(p) = app_data_override {
            p
        } else {
            dirs::data_dir()
                .map(|d| d.join("zorter"))
                .unwrap_or_else(|| PathBuf::from("./data"))
        };

        let config_dir = base_dir.join("config");
        let index_dir = base_dir.join("index");
        let embeddings_dir = index_dir.join("embeddings");
        let history_dir = base_dir.join("history");
        let cache_dir = base_dir.join("cache").join("thumbnails");

        fs::create_dir_all(&config_dir).map_err(AppError::Io)?;
        fs::create_dir_all(&index_dir).map_err(AppError::Io)?;
        fs::create_dir_all(&embeddings_dir).map_err(AppError::Io)?;
        fs::create_dir_all(&history_dir).map_err(AppError::Io)?;
        fs::create_dir_all(&cache_dir).map_err(AppError::Io)?;

        let settings_path = config_dir.join("settings.json");
        let settings: AppSettings = recover_or_load(&settings_path)?;

        let files_path = index_dir.join("files.json");
        let files_index: FilesIndex = recover_or_load(&files_path)?;
        let mut files_map = HashMap::new();
        for f in files_index.files {
            files_map.insert(f.id.clone(), f);
        }

        let analysis_path = index_dir.join("analysis.json");
        let analysis_index: AnalysisIndex = recover_or_load(&analysis_path)?;
        let mut analysis_map = HashMap::new();
        for a in analysis_index.analyses {
            analysis_map.insert(a.file_id.clone(), a);
        }

        let dirs_path = index_dir.join("directories.json");
        let dirs_index: DirectoriesIndex = recover_or_load(&dirs_path)?;

        // Load embeddings shards
        let mut embeddings_map = HashMap::new();
        let mut embedding_model = "embeddinggemma-2:270m".to_string();
        let mut embedding_dims = 0;

        if let Ok(entries) = fs::read_dir(&embeddings_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().is_some_and(|ext| ext == "json")
                    && let Ok(shard) = read_json_file::<EmbeddingShard>(&p) {
                        embedding_model = shard.model;
                        embedding_dims = shard.dimensions;
                        for emb in shard.embeddings {
                            embeddings_map.insert(emb.file_id, emb.vector);
                        }
                    }
            }
        }

        let journal_path = history_dir.join("operations.jsonl");

        Ok(Self {
            base_dir,
            settings,
            files: files_map,
            analyses: analysis_map,
            embeddings: embeddings_map,
            embedding_model,
            embedding_dims,
            directories: dirs_index.directories,
            journal_path,
        })
    }

    pub fn get_base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn get_cache_dir(&self) -> PathBuf {
        self.base_dir.join("cache")
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.clone()
    }

    pub fn set_vision_model(&mut self, model: &str) -> Result<()> {
        self.settings.vision_model = model.to_string();
        write_json_atomic(&self.base_dir.join("config").join("settings.json"), &self.settings)
    }


    pub fn get_all_files(&self) -> Vec<IndexedFile> {
        self.files.values().cloned().collect()
    }

    pub fn get_file(&self, file_id: &str) -> Option<IndexedFile> {
        self.files.get(file_id).cloned()
    }

    pub fn save_files(&mut self, files: &[IndexedFile]) -> Result<()> {
        for f in files {
            self.files.insert(f.id.clone(), f.clone());
        }
        let index = FilesIndex {
            version: 1,
            files: self.files.values().cloned().collect(),
        };
        let path = self.base_dir.join("index").join("files.json");
        write_json_atomic(&path, &index)?;
        Ok(())
    }

    pub fn get_all_analyses(&self) -> Vec<FileAnalysis> {
        self.analyses.values().cloned().collect()
    }

    pub fn get_analysis(&self, file_id: &str) -> Option<FileAnalysis> {
        self.analyses.get(file_id).cloned()
    }

    pub fn save_analyses(&mut self, analyses: &[FileAnalysis]) -> Result<()> {
        for a in analyses {
            self.analyses.insert(a.file_id.clone(), a.clone());
        }
        let index = AnalysisIndex {
            version: 1,
            analyses: self.analyses.values().cloned().collect(),
        };
        let path = self.base_dir.join("index").join("analysis.json");
        write_json_atomic(&path, &index)?;
        Ok(())
    }


    pub fn get_all_embeddings(&self) -> HashMap<String, Vec<f32>> {
        self.embeddings.clone()
    }

    pub fn save_embeddings(
        &mut self,
        model: &str,
        dimensions: usize,
        new_embeddings: &[(String, Vec<f32>)],
    ) -> Result<()> {
        self.embedding_model = model.to_string();
        self.embedding_dims = dimensions;

        for (file_id, vec) in new_embeddings {
            self.embeddings.insert(file_id.clone(), vec.clone());
        }

        // Shard embeddings into chunks of 500 vectors per shard
        let shard_size = 500;
        let mut all_pairs: Vec<FileEmbedding> = self
            .embeddings
            .iter()
            .map(|(k, v)| FileEmbedding {
                file_id: k.clone(),
                vector: v.clone(),
            })
            .collect();

        all_pairs.sort_by(|a, b| a.file_id.cmp(&b.file_id));

        let embeddings_dir = self.base_dir.join("index").join("embeddings");
        for (idx, chunk) in all_pairs.chunks(shard_size).enumerate() {
            let shard_file = embeddings_dir.join(format!("embeddings_{:03}.json", idx + 1));
            let shard = EmbeddingShard {
                version: 1,
                shard_id: idx + 1,
                model: model.to_string(),
                dimensions,
                embeddings: chunk.to_vec(),
            };
            write_json_atomic(&shard_file, &shard)?;
        }

        Ok(())
    }


    pub fn add_monitored_directory(&mut self, dir_path: &str, total_files: usize) -> Result<()> {
        if !self.directories.iter().any(|d| d.path == dir_path) {
            self.directories.push(MonitoredDirectory {
                path: dir_path.to_string(),
                added_at: chrono::Utc::now().to_rfc3339(),
                total_files,
            });
            let index = DirectoriesIndex {
                version: 1,
                directories: self.directories.clone(),
            };
            let path = self.base_dir.join("index").join("directories.json");
            write_json_atomic(&path, &index)?;
        }
        Ok(())
    }

    pub fn append_operation_journal(&self, entry: &OperationJournalEntry) -> Result<()> {
        append_jsonl(&self.journal_path, entry)
    }

    pub fn get_operation_history(&self) -> Result<Vec<OperationJournalEntry>> {
        if !self.journal_path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.journal_path).map_err(AppError::Io)?;
        let mut entries = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<OperationJournalEntry>(line) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    pub fn apply_operation_updates(&mut self, entry: &OperationJournalEntry) -> Result<()> {
        let is_rollback = entry.operation_type.starts_with("rollback:");

        for action in &entry.actions {
            if let Some(file) = self.files.get_mut(&action.file_id) {
                if is_rollback {
                    file.current_path = action.source_path.clone();
                    file.current_name = Path::new(&action.source_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&file.current_name)
                        .to_string();
                } else {
                    file.current_path = action.destination_path.clone();
                    file.current_name = Path::new(&action.destination_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&file.current_name)
                        .to_string();
                }
            }
        }

        let all_files = self.get_all_files();
        self.save_files(&all_files)?;
        self.append_operation_journal(entry)?;
        Ok(())
    }
}
