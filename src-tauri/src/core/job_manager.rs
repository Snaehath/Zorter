use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct JobManager {
    cancellation_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    running_jobs: Arc<Mutex<HashMap<String, String>>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            cancellation_flags: Arc::new(Mutex::new(HashMap::new())),
            running_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_job(&self, job_id: &str, description: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        let mut flags = self.cancellation_flags.lock().unwrap();
        flags.insert(job_id.to_string(), flag.clone());

        let mut jobs = self.running_jobs.lock().unwrap();
        jobs.insert(job_id.to_string(), description.to_string());

        flag
    }

    pub fn cancel_job(&self, job_id: &str) -> bool {
        let flags = self.cancellation_flags.lock().unwrap();
        if let Some(flag) = flags.get(job_id) {
            flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn finish_job(&self, job_id: &str) {
        let mut flags = self.cancellation_flags.lock().unwrap();
        flags.remove(job_id);

        let mut jobs = self.running_jobs.lock().unwrap();
        jobs.remove(job_id);
    }

    pub fn is_cancelled(flag: &Arc<AtomicBool>) -> bool {
        flag.load(Ordering::SeqCst)
    }

    pub fn get_running_jobs(&self) -> HashMap<String, String> {
        self.running_jobs.lock().unwrap().clone()
    }
}
