use std::sync::{Arc, Mutex};
use crate::ai::ollama::OllamaClient;
use crate::ai::vision::VisionCoordinator;
use crate::core::job_manager::JobManager;
use crate::search::semantic::SemanticSearchEngine;
use crate::storage::repository::Repository;

pub struct AppState {
    pub repository: Arc<Mutex<Repository>>,
    pub ollama: Arc<OllamaClient>,
    pub vision_coordinator: Arc<VisionCoordinator>,
    pub search_engine: Arc<SemanticSearchEngine>,
    pub job_manager: JobManager,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let repo = Repository::new(None).expect("Failed to initialize repository");
        let settings = repo.get_settings();
        let ollama = Arc::new(OllamaClient::new(&settings.ollama_endpoint));

        let vision_coordinator = Arc::new(VisionCoordinator::new(ollama.clone()));

        let search_engine = Arc::new(SemanticSearchEngine::new(ollama.clone()));

        Self {
            repository: Arc::new(Mutex::new(repo)),
            ollama,
            vision_coordinator,
            search_engine,
            job_manager: JobManager::new(),
        }
    }
}
