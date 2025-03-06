use notify::Result;
use std::path::PathBuf;
use structopt::StructOpt;
use std::process;

use file_watcher::{
    watcher::FileWatcher,
    service::detector::ServiceDetector,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "file-watcher")]
struct Opt {
    /// Paths to watch for changes
    #[structopt(long = "paths", required = true, parse(from_os_str))]
    paths: Vec<PathBuf>,

    /// Command to run when files change
    #[structopt(last = true)]
    command: Vec<String>,

    /// Services configuration file
    #[structopt(long = "services-config")]
    services_config: Option<PathBuf>,

    /// Output format for affected services
    #[structopt(long = "service-format", default_value = "name")]
    service_format: ServiceFormat,
}

#[derive(Debug)]
enum ServiceFormat {
    Name,
    NamePath,
}

impl std::str::FromStr for ServiceFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "name" => Ok(ServiceFormat::Name),
            "name-path" => Ok(ServiceFormat::NamePath),
            _ => Err("Invalid service format".into()),
        }
    }
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    
    // Load services configuration
    let service_configs = if let Some(config_path) = &opt.services_config {
        let content = std::fs::read_to_string(config_path)?;
        toml::from_str(&content).map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse services config: {}", e)
        ))?
    } else {
        Vec::new()
    };

    // Detect services
    let detector = ServiceDetector::new(service_configs);
    let services = detector.detect_services(&opt.paths[0])?;

    let mut watcher = FileWatcher::new(&opt.paths, services)?;
    watcher.watch(&opt.paths)?;

    println!("Watching paths: {:?}", opt.paths);
    println!("Will run command: {:?}", opt.command);
    println!("Listening! Ctrl-C to quit.");

    loop {
        if let Err(e) = watcher.handle_events(&opt.command) {
            eprintln!("Error handling events: {}", e);
        }

        // Print affected services
        for (name, path) in watcher.get_affected_services() {
            match opt.service_format {
                ServiceFormat::Name => println!("{}", name),
                ServiceFormat::NamePath => println!("{},{}", name, path.display()),
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
