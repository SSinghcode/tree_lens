use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "git")]
use std::sync::Arc;
#[cfg(feature = "git")]
use git2::{Repository, Status};



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GitStatus {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Ignored,
    Clean,
}

impl GitStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            GitStatus::Untracked => "?",
            GitStatus::Modified  => "M",
            GitStatus::Added     => "A",
            GitStatus::Deleted   => "D",
            GitStatus::Renamed   => "R",
            GitStatus::Ignored   => "I",
            GitStatus::Clean     => " ",
        }
    }

    pub fn color(&self) -> colored::Color {
        match self {
            GitStatus::Untracked => colored::Color::BrightRed,
            GitStatus::Modified  => colored::Color::Yellow,
            GitStatus::Added     => colored::Color::Green,
            GitStatus::Deleted   => colored::Color::Red,
            GitStatus::Renamed   => colored::Color::Cyan,
            GitStatus::Ignored   => colored::Color::BrightBlack,
            GitStatus::Clean     => colored::Color::White,
        }
    }
}

#[derive(Clone)]
pub struct GitInfo {
    #[cfg(feature = "git")]
    repo: Option<Arc<Repository>>,
    status_cache: HashMap<PathBuf, GitStatus>,
}

impl GitInfo {
    /// Opens the git repo at or above `path` and stores a shared handle
    pub fn new(path: &Path) -> Self {
        #[cfg(feature = "git")]
        {
            let repo = Repository::discover(path).ok();
            let mut git_info = Self {
                repo: repo.map(Arc::new),
                status_cache: HashMap::new(),
            };
            git_info.load_status();
            return git_info;
        }
        #[cfg(not(feature = "git"))]
        {
            let _ = path;
            Self { status_cache: HashMap::new() }
        }
    }

    #[cfg(feature = "git")]
    fn load_status(&mut self) {
        if let Some(ref repo) = self.repo {
            let mut status_opts = git2::StatusOptions::new();
            status_opts.include_ignored(true);
            status_opts.include_untracked(true);

            if let Ok(statuses) = repo.statuses(Some(&mut status_opts)) {
                for entry in statuses.iter() {
                    if let Some(path) = entry.path() {
                        let path_buf = PathBuf::from(path);
                        let status = self.convert_status(entry.status());
                        self.status_cache.insert(path_buf, status);
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "git"))]
    fn load_status(&mut self) {
        // no-op for non-git builds
    }

    #[cfg(feature = "git")]
    fn convert_status(&self, flags: Status) -> GitStatus {
        // check index status first (staged changes)
        if flags.contains(Status::INDEX_NEW) {
            GitStatus::Added
        } else if flags.contains(Status::INDEX_MODIFIED) {
            GitStatus::Modified
        } else if flags.contains(Status::INDEX_DELETED) {
            GitStatus::Deleted
        } else if flags.contains(Status::INDEX_RENAMED) {
            GitStatus::Renamed
        // worktree status (unstaged changes)
        } else if flags.contains(Status::WT_NEW) {
            GitStatus::Untracked
        } else if flags.contains(Status::WT_MODIFIED) {
            GitStatus::Modified
        } else if flags.contains(Status::WT_DELETED) {
            GitStatus::Deleted
        } else if flags.contains(Status::WT_RENAMED) {
            GitStatus::Renamed
        } else if flags.contains(Status::IGNORED) {
            GitStatus::Ignored
        } else {
            GitStatus::Clean
        }
    }

    pub fn get_status(&self, path: &Path) -> GitStatus {
        #[cfg(feature = "git")]
        if let Some(ref repo) = self.repo {
            if let Some(workdir) = repo.workdir() {
                if let Ok(relative_path) = path.strip_prefix(workdir) {
                    let relative_path = relative_path.strip_prefix("./").unwrap_or(relative_path);
                    if let Some(status) = self.status_cache.get(relative_path) {
                        return *status;
                    }
                    // Normalize path separators (Windows uses `\`, git uses `/`)
                    let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
                    let normalized_path = PathBuf::from(relative_path_str);
                    if let Some(status) = self.status_cache.get(&normalized_path) {
                        return *status;
                    }
                }
            }
        }

        // Fallback: try the path as-is (e.g. already relative)
        if let Some(status) = self.status_cache.get(path) {
            return *status;
        }

        GitStatus::Clean
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        matches!(self.get_status(path), GitStatus::Ignored)
    }
}
