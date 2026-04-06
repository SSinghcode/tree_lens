use regex::Regex;
use std::{path::Path};

pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("."))
        .unwrap_or(false)
}

pub fn get_file_extensions(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

pub fn format_permissions(mode: u32) -> String {
    let mut perms = String::new();

    // owner permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // other permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    perms
}

pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
    if let Ok(regex) = Regex::new(pattern) {
        if let Some(path_str) = path.to_str() {
            return regex.is_match(path_str);
        }
    }

    // Fallback to simple glob-like matching
    let path_str = path.to_string_lossy();
    let pattern = pattern.replace("*", ".*").replace("?", ".");
    if let Ok(regex) = Regex::new(&format!("^{}$", pattern)) {
        return regex.is_match(&path_str);
    }

    false
}

pub fn calculate_md5(path: &Path) -> crate::Result<String> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    let digest = md5::compute(&contents);
    Ok(format!("{:x}", digest))
}

pub fn format_time(timestamp: std::time::SystemTime) -> String {
    use chrono::{DateTime, Local};
    let datetime: DateTime<Local> = timestamp.into();
    datetime.format("%Y-%m-%d  %H:%M:%S").to_string()
}

pub fn count_files_in_dir(path: &Path) -> (usize, usize) {
    let mut file_count = 0;
    let mut dir_count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    dir_count += 1;
                } else {
                    file_count += 1;
                }
            }
        }
    }
    (file_count, dir_count)
}

pub fn parse_size(size_str: &Option<String>) -> crate::Result<Option<u64>> {
    match size_str {
        Some(s) => {
            let upper = s.to_uppercase();
            let (number_part, unit) = if upper.ends_with("GB") {
                (upper.trim_end_matches("GB"), 1_000_000_000)
            } else if upper.ends_with("MB") {
                (upper.trim_end_matches("MB"), 1_000_000)
            } else if upper.ends_with("KB") {
                (upper.trim_end_matches("KB"), 1_000)
            } else if upper.ends_with("B") {
                (upper.trim_end_matches("B"), 1)
            } else {
                return Err("Invalid size format".into());
            };

            let number = number_part.trim().parse::<u64>()?;
            Ok(Some(number * unit))
        }
        None => Ok(None),
    }
}
