use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use crate::core::errors::{AppError, Result};

pub struct FolderWatcher {
    _watcher: RecommendedWatcher,
}

impl FolderWatcher {
    pub fn watch<F>(directories: &[PathBuf], on_event: F) -> Result<Self>
    where
        F: Fn(Event) + Send + 'static,
    {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| AppError::Generic(format!("Watcher init error: {}", e)))?;

        for dir in directories {
            if dir.exists() && dir.is_dir() {
                let _ = watcher.watch(dir, RecursiveMode::Recursive);
            }
        }

        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                on_event(event);
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
