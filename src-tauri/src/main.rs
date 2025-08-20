#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::{command, State};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Person {
    name: String,
    dob: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Household {
    id: u32,
    household_name: String,
    persons: Vec<Person>,
    next_review_due: String,
    review_type: String, // "Required" or "Periodic"
    auc: f64,
    segment: String, // "Black", "Green", "Yellow", "Red"
    last_review_date: Option<String>,
    review_status: String, // "Scheduled", "Completed", "Overdue"
    priority_flag: String,
    assigned_month: Option<String>,
    created: String,
    updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    households: Vec<Household>,
    settings: AppSettings,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    last_file_path: Option<String>,
    theme: String, // "light" or "dark"
    auto_backup: bool,
    backup_count: u32,
}

impl Default for AppData {
    fn default() -> Self {
        AppData {
            households: Vec::new(),
            settings: AppSettings {
                last_file_path: None,
                theme: "light".to_string(),
                auto_backup: true,
                backup_count: 10,
            },
            version: "1.0.0".to_string(),
        }
    }
}

#[command]
fn load_data_file(path: String) -> Result<AppData, String> {
    if !Path::new(&path).exists() {
        return Ok(AppData::default());
    }
    
    match fs::read_to_string(&path) {
        Ok(content) => {
            // Handle both AppData format and simple CSV text for export
            if content.starts_with('{') {
                match serde_json::from_str::<AppData>(&content) {
                    Ok(data) => Ok(data),
                    Err(e) => Err(format!("Failed to parse JSON: {}", e)),
                }
            } else {
                // Return error for non-JSON files
                Err("File is not a valid JSON data file".to_string())
            }
        }
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

#[command]
fn save_data_file(path: String, data: serde_json::Value) -> Result<(), String> {
    // Check if data is a string (CSV export) or AppData object
    let content = if data.is_string() {
        data.as_str().unwrap().to_string()
    } else {
        // Serialize as JSON for AppData
        match serde_json::to_string_pretty(&data) {
            Ok(json) => json,
            Err(e) => return Err(format!("Failed to serialize data: {}", e)),
        }
    };
    
    match fs::write(&path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write file: {}", e)),
    }
}

#[command]
fn create_backup(path: String, data: AppData) -> Result<String, String> {
    let file_path = Path::new(&path);
    let file_stem = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("backup");
    let parent_dir = file_path.parent().unwrap_or(Path::new("."));
    
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S");
    let backup_name = format!("{}_backup_{}.json", file_stem, timestamp);
    let backup_path = parent_dir.join(backup_name);
    
    match serde_json::to_string_pretty(&data) {
        Ok(json) => {
            match fs::write(&backup_path, json) {
                Ok(_) => Ok(backup_path.to_string_lossy().to_string()),
                Err(e) => Err(format!("Failed to create backup: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to serialize backup data: {}", e)),
    }
}

#[command]
fn cleanup_old_backups(directory: String, file_stem: String, keep_count: u32) -> Result<(), String> {
    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Ok(());
    }

    let mut backups = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&format!("{}_backup_", file_stem)) && file_name.ends_with(".json") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(created) = metadata.created() {
                            backups.push((entry.path(), created));
                        }
                    }
                }
            }
        }
    }

    // Sort by creation time, newest first
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    // Remove old backups
    for (path, _) in backups.iter().skip(keep_count as usize) {
        let _ = fs::remove_file(path);
    }

    Ok(())
}

#[command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[command]
fn validate_file_path(path: String) -> Result<bool, String> {
    let file_path = Path::new(&path);
    
    // Check if path is valid
    if !file_path.is_absolute() {
        return Err("Path must be absolute".to_string());
    }
    
    // Check if parent directory exists
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            return Err("Parent directory does not exist".to_string());
        }
    }
    
    // Check if file exists (for reading)
    Ok(file_path.exists())
}

#[command]
fn create_directory(path: String) -> Result<(), String> {
    match fs::create_dir_all(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to create directory: {}", e)),
    }
}

#[command]
fn get_file_info(path: String) -> Result<FileInfo, String> {
    let file_path = Path::new(&path);
    
    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }
    
    match file_path.metadata() {
        Ok(metadata) => {
            let size = metadata.len();
            let modified = metadata.modified()
                .map_err(|e| format!("Failed to get modification time: {}", e))?;
            
            let modified_datetime: DateTime<Utc> = modified.into();
            
            Ok(FileInfo {
                size,
                modified: modified_datetime.to_rfc3339(),
                is_readonly: metadata.permissions().readonly(),
            })
        }
        Err(e) => Err(format!("Failed to get file metadata: {}", e)),
    }
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    size: u64,
    modified: String,
    is_readonly: bool,
}

#[command]
fn export_backup_list(directory: String, file_stem: String) -> Result<Vec<BackupInfo>, String> {
    let dir_path = Path::new(&directory);
    if !dir_path.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(&format!("{}_backup_", file_stem)) && file_name.ends_with(".json") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(created) = metadata.created() {
                            let created_datetime: DateTime<Utc> = created.into();
                            backups.push(BackupInfo {
                                filename: file_name,
                                path: entry.path().to_string_lossy().to_string(),
                                created: created_datetime.to_rfc3339(),
                                size: metadata.len(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by creation time, newest first
    backups.sort_by(|a, b| b.created.cmp(&a.created));

    Ok(backups)
}

#[derive(Debug, Serialize)]
pub struct BackupInfo {
    filename: String,
    path: String,
    created: String,
    size: u64,
}

#[command]
fn restore_backup(backup_path: String, target_path: String) -> Result<(), String> {
    if !Path::new(&backup_path).exists() {
        return Err("Backup file does not exist".to_string());
    }
    
    match fs::copy(&backup_path, &target_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to restore backup: {}", e)),
    }
}

#[command]
fn delete_backup(backup_path: String) -> Result<(), String> {
    if !Path::new(&backup_path).exists() {
        return Err("Backup file does not exist".to_string());
    }
    
    match fs::remove_file(&backup_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to delete backup: {}", e)),
    }
}

#[command]
fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    os: String,
    arch: String,
    app_version: String,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_data_file,
            save_data_file,
            create_backup,
            cleanup_old_backups,
            get_app_version,
            validate_file_path,
            create_directory,
            get_file_info,
            export_backup_list,
            restore_backup,
            delete_backup,
            get_system_info
        ])
        .setup(|app| {
            // Set up any initial configuration here
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
