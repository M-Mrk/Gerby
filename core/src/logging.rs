use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const LOG_DIR_NAME: &str = "core_logs";
const MAX_LOGS: u32 = 255;

fn get_log_dir() -> PathBuf {
    let mut path = std::env::current_exe().expect("Failed to find executable path!");

    // remove executable name
    path.pop();

    // add dir name
    path.push(LOG_DIR_NAME);

    path
}

fn get_num_logs_stored() -> u32 {
    match fs::read_dir(get_log_dir()) {
        Ok(logs) => logs.filter_map(Result::ok).count() as u32,
        Err(err) => {
            eprintln!("Exiting, failed to read number of stored logs!: {}", err);
            std::process::exit(1);
        }
    }
}

fn prune_log() {
    let mut logs: Vec<PathBuf> = fs::read_dir(get_log_dir())
        .unwrap_or_else(|_| {
            eprintln!("Exiting, failed to read stored logs!");
            std::process::exit(1);
        })
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                name.starts_with("gerby_cli-") && name.ends_with(".log")
            } else {
                false
            }
        })
        .collect();

    logs.sort();

    if let Some(oldest) = logs.first() {
        eprintln!("Pruning a log: {}!", oldest.display());
        fs::remove_file(oldest).unwrap_or_else(|err| {
            eprintln!("Exiting, failed to prune log!: {}", err);
            std::process::exit(1);
        });
    }
}

/// Sets up logging to a file and stderr.
///
/// If the max of logs is hit, oldest logs get pruned to make space for the new one
pub fn init() {
    // Make sure log dir exists
    fs::create_dir_all(get_log_dir()).unwrap_or_else(|err| {
        eprintln!(
            "Exiting, failed to create {:?} dir!: {}",
            get_log_dir().as_path(),
            err
        );
        std::process::exit(1);
    });

    let num_logs = get_num_logs_stored();
    if num_logs >= MAX_LOGS {
        // remove enough logs to have space for the new one by this execution
        for _ in 0..(num_logs - MAX_LOGS + 1) {
            prune_log()
        }
    }

    let execution_time = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file_name = format!("gerby_cli-{}.log", execution_time);

    let log_file_path = Path::new(get_log_dir().as_path()).join(log_file_name);
    let log_file = File::create(log_file_path).unwrap_or_else(|err| {
        eprintln!("Exiting, failed to create log file!: {}", err);
        std::process::exit(1);
    });

    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let stderr_output = tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .with_ansi(true)
        .pretty();

    let file_output = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_output)
        .with(file_output)
        .init();
}
