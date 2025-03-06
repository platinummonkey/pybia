use crate::dependency::DependencyGraph;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use std::collections::HashSet;
use std::collections::HashMap;
use crate::service::models::DetectedService;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<notify::Event>,
    _tx: Sender<notify::Event>,
    dependency_graph: DependencyGraph,
    last_run: Instant,
    debounce_duration: Duration,
    last_changed_file: Option<PathBuf>,
}

impl FileWatcher {
    pub fn new(paths: &[PathBuf], services: HashMap<String, DetectedService>) -> notify::Result<Self> {
        let (tx, rx) = channel();
        let tx_clone = tx.clone();
        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx_clone.send(event);
                }
            },
            Config::default(),
        )?;

        let mut dependency_graph = DependencyGraph::new();
        for path in paths {
            dependency_graph.build_from_directory(path, services.clone())?;
        }

        Ok(FileWatcher {
            watcher,
            rx,
            _tx: tx,
            dependency_graph,
            last_run: Instant::now(),
            debounce_duration: Duration::from_millis(100),
            last_changed_file: None,
        })
    }

    pub fn watch(&mut self, paths: &[PathBuf]) -> notify::Result<()> {
        for path in paths {
            self.watcher.watch(path, RecursiveMode::Recursive)?;
        }
        Ok(())
    }

    pub fn get_affected_services(&self) -> Vec<(&str, &Path)> {
        self.last_changed_file.as_ref()
            .map(|f| self.dependency_graph.get_affected_services(f))
            .unwrap_or_default()
    }

    pub fn handle_events(&mut self, command: &[String]) -> notify::Result<()> {
        let mut processed_paths = HashSet::new();

        while let Ok(event) = self.rx.try_recv() {
            for path in event.paths {
                if !processed_paths.contains(&path) {
                    processed_paths.insert(path.clone());
                    println!("\nChanged path: {}", path.display());
                    
                    if let Err(e) = self.run_command(command, &path) {
                        eprintln!("Failed to run command: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    fn run_command(&mut self, command: &[String], changed_file: &Path) -> std::io::Result<bool> {
        let now = Instant::now();
        if now.duration_since(self.last_run) < self.debounce_duration {
            return Ok(false);
        }
        self.last_run = now;
        self.last_changed_file = Some(changed_file.to_path_buf());

        let status = std::process::Command::new(&command[0])
            .args(&command[1..])
            .status()?;

        Ok(status.success())
    }
} 