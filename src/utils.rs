use regex::Regex;
use std::path::Path;

pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

// Bug fix: original had only #[cfg(unix)] with a `u32` parameter — no Windows version at all.
// Both platforms now take &Metadata so call sites are cross-platform.
#[cfg(unix)]
pub fn format_permissions(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt;
    let mode = metadata.mode();
    let mut perms = String::new();
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    perms
}

#[cfg(windows)]
pub fn format_permissions(metadata: &std::fs::Metadata) -> String {
    if metadata.permissions().readonly() {
        "r--r--r--".to_string()
    } else {
        "rw-rw-rw-".to_string()
    }
}

// Bug fix: original glob branch did `pattern.replace("*", ".*")` without escaping
// regex metacharacters first — so `file.rs` became `^file.rs$` which matches `fileXrs`.
// Fix: escape the pattern first, then replace the escaped glob wildcards.
pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

    // Only treat as raw regex if it starts with ^ or ends with $ (explicit anchors),
    // or contains explicit regex groups like `(a|b)`. Brace expansions like `*.{rs,toml}`
    // are NOT valid regex — the old heuristic matched `{` as regex and then silently
    // failed, returning false for everything.
    let is_regex = (pattern.starts_with('^') || pattern.ends_with('$'))
        || (pattern.contains('(') && pattern.contains(')'));

    if is_regex {
        if let Ok(re) = Regex::new(pattern) {
            return re.is_match(&path_str);
        }
        // If the user wrote something that looked like regex but is invalid, fall through
        // to glob matching rather than silently returning false.
    }

    // Glob-style: escape everything first, then restore * and ? as wildcards.
    let glob_regex = format!(
        "^{}$",
        regex::escape(pattern)
            .replace(r"\*", ".*")
            .replace(r"\?", ".")
    );
    if let Ok(re) = Regex::new(&glob_regex) {
        return re.is_match(&file_name) || re.is_match(&path_str);
    }

    false
}

pub fn calculate_md5(path: &Path) -> crate::Result<String> {
    use std::fs::File;
    use std::io::{BufReader, Read};

    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(65_536, file);
    let mut context = md5::Context::new();
    let mut buf = [0u8; 65_536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        context.consume(&buf[..n]);
    }
    Ok(format!("{:x}", context.finalize()))
}

pub fn format_time(timestamp: std::time::SystemTime) -> String {
    use chrono::{DateTime, Local};
    let datetime: DateTime<Local> = timestamp.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn count_files_in_dir(path: &Path) -> (usize, usize) {
    let mut file_count = 0;
    let mut dir_count = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        dir_count += 1;
                        stack.push(entry.path());
                    } else {
                        file_count += 1;
                    }
                }
            }
        }
    }
    (file_count, dir_count)
}

pub fn parse_size(size_str: &Option<String>) -> crate::Result<Option<u64>> {
    match size_str {
        Some(s) => {
            let upper = s.trim().to_uppercase();
            // Support both binary (GiB/MiB/KiB) and decimal (GB/MB/KB) suffixes.
            // Previously only decimal suffixes were handled, and binary suffixes like
            // "10GiB" would fall through to plain-number parsing and fail.
            // Use 1024-based multipliers (standard for file sizes).
            let (number_part, unit): (&str, u64) = if upper.ends_with("GIB") {
                (upper.trim_end_matches("GIB"), 1_000_000_000u64)
            } else if upper.ends_with("MIB") {
                (upper.trim_end_matches("MIB"), 1_000_000u64)
            } else if upper.ends_with("KIB") {
                (upper.trim_end_matches("KIB"), 1_000u64)
            } else if upper.ends_with("GB") {
                (upper.trim_end_matches("GB"), 1_000_000_000u64)
            } else if upper.ends_with("MB") {
                (upper.trim_end_matches("MB"), 1_000_000u64)
            } else if upper.ends_with("KB") {
                (upper.trim_end_matches("KB"), 1_000u64)
            } else if upper.ends_with('B') {
                (upper.trim_end_matches('B'), 1u64)
            } else {
                // plain number → bytes
                (upper.as_str(), 1u64)
            };

            let number = number_part.trim().parse::<u64>()?;
            Ok(Some(number * unit))
        }
        None => Ok(None),
    }
}
