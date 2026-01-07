use crate::settings::DEFAULT_LOG_FILE_NAME;
use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, RwLock};

const LOG_ROTATE_BYTES: u64 = 100 * 1024;

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: LevelFilter,
    pub file_path: PathBuf,
}

impl LoggingConfig {
    pub fn new(enabled: bool, level: LevelFilter, file_path: PathBuf) -> Self {
        Self { enabled, level, file_path }
    }
}

struct FileState {
    path: PathBuf,
    file: Option<File>,
    size: u64,
}

impl FileState {
    fn open(path: PathBuf) -> io::Result<Self> {
        let (file, size) = open_log_file(&path, true)?;
        Ok(Self { path, file: Some(file), size })
    }

    fn rotate_if_needed(&mut self, next_len: u64) -> io::Result<()> {
        if self.size + next_len <= LOG_ROTATE_BYTES {
            return Ok(());
        }

        self.close();
        rotate_log_file(&self.path)?;
        self.open_new()?;
        Ok(())
    }

    fn close(&mut self) {
        self.file = None;
    }

    fn open_new(&mut self) -> io::Result<()> {
        let (file, _) = open_log_file(&self.path, false)?;
        self.file = Some(file);
        self.size = 0;
        Ok(())
    }
}

struct Logger {
    config: RwLock<LoggingConfig>,
    file_state: Mutex<Option<FileState>>,
}

impl Logger {
    fn new(config: LoggingConfig) -> Self {
        Self { config: RwLock::new(config), file_state: Mutex::new(None) }
    }

    fn set_config(&self, config: LoggingConfig) {
        let mut guard = self.config.write().expect("logger config lock poisoned");
        let path_changed = guard.file_path != config.file_path;
        let enabled_changed = guard.enabled != config.enabled;
        *guard = config;

        if path_changed || enabled_changed {
            let mut file_state = self.file_state.lock().expect("logger file state lock poisoned");
            *file_state = None;
        }
    }

    fn max_level(&self) -> LevelFilter {
        let config = self.config.read().expect("logger config lock poisoned");
        if config.enabled { config.level } else { LevelFilter::Off }
    }

    fn is_app_target(target: &str) -> bool {
        target == "oxide_mongo" || target.starts_with("oxide_mongo::")
    }

    fn write_line(&self, line: &str, config: &LoggingConfig) {
        let _ = io::stderr().write_all(line.as_bytes());

        let mut guard = self.file_state.lock().expect("logger file state lock poisoned");
        let state = match guard.as_mut() {
            Some(state) if state.path == config.file_path => state,
            _ => {
                *guard = FileState::open(config.file_path.clone()).ok();
                match guard.as_mut() {
                    Some(state) => state,
                    None => return,
                }
            }
        };

        let bytes = line.as_bytes();
        if state.rotate_if_needed(bytes.len() as u64).is_err() {
            *guard = None;
            return;
        }

        if let Some(file) = state.file.as_mut() {
            if file.write_all(bytes).is_ok() {
                state.size = state.size.saturating_add(bytes.len() as u64);
            } else {
                *guard = None;
            }
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let config = self.config.read().expect("logger config lock poisoned");
        config.enabled && metadata.level() <= config.level && Self::is_app_target(metadata.target())
    }

    fn log(&self, record: &Record) {
        let config = self.config.read().expect("logger config lock poisoned").clone();
        if !config.enabled || record.level() > config.level || !Self::is_app_target(record.target())
        {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!(
            "{timestamp} [{level}] {message}\n",
            level = record.level(),
            message = record.args()
        );
        self.write_line(&line, &config);
    }

    fn flush(&self) {}
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn apply_settings(enabled: bool, level: LevelFilter, file_path: &str) {
    let trimmed = file_path.trim();
    let path = if trimmed.is_empty() {
        PathBuf::from(DEFAULT_LOG_FILE_NAME)
    } else {
        PathBuf::from(trimmed)
    };

    let config = LoggingConfig::new(enabled, level, path);
    let logger = LOGGER.get_or_init(|| Logger::new(config.clone()));
    logger.set_config(config);
    let _ = log::set_logger(logger);
    log::set_max_level(logger.max_level());
}

fn open_log_file(path: &Path, append: bool) -> io::Result<(File, u64)> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let file =
        OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(path)?;
    let size = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    Ok((file, size))
}

fn rotate_log_file(path: &Path) -> io::Result<()> {
    let rotated = rotated_log_path(path);
    if rotated.exists() {
        let _ = fs::remove_file(&rotated);
    }
    if path.exists() {
        fs::rename(path, rotated)?;
    }
    Ok(())
}

fn rotated_log_path(path: &Path) -> PathBuf {
    let file_name =
        path.file_name().and_then(|name| name.to_str()).unwrap_or(DEFAULT_LOG_FILE_NAME);
    path.with_file_name(format!("{file_name}.1"))
}
