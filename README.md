# tree_lens
A powerful CLI tool to visualize and analyze directory structures with Git integration, filtering, and multiple output formats.

## Project Status
Currently in development. Core configuration layer is complete.

## What's Built So Far

### `src/config.rs`
The brain of the application — stores and manages all settings.

**Enums:**
- `SortBy` — how to sort files: `name`, `size`, `time`, `extension`, `type`
- `OutputFormat` — how to display output: `tree`, `json`, `xml`, `csv`, `md`

**`Config` struct — all available settings:**
| Field | Type | Description |
|---|---|---|
| `max_depth` | `Option<usize>` | How many folder levels deep to walk |
| `show_hidden` | `bool` | Show hidden files (starting with `.`) |
| `show_size` | `bool` | Show file sizes |
| `show_permission` | `bool` | Show file permissions |
| `show_time` | `bool` | Show last modified time |
| `show_count` | `bool` | Show file/folder count per directory |
| `use_colors` | `bool` | Colorful terminal output |
| `sort_by` | `SortBy` | Sort order |
| `reverse_sort` | `bool` | Reverse the sort order |
| `filter_extension` | `Option<String>` | Only show files with these extensions e.g. `"rs,toml"` |
| `directories_only` | `bool` | Show only folders |
| `files_only` | `bool` | Show only files |
| `min_size` | `Option<u64>` | Minimum file size in bytes |
| `max_size` | `Option<u64>` | Maximum file size in bytes |
| `exclude_patterns` | `Vec<String>` | Patterns to hide |
| `include_patterns` | `Vec<String>` | Patterns to show |
| `git_ignore` | `bool` | Respect `.gitignore` |
| `git_status` | `bool` | Show git status next to files |
| `limit` | `Option<usize>` | Max number of files to show |
| `output_format` | `OutputFormat` | Output format |
| `follow_links` | `bool` | Follow symbolic links |
| `full_path` | `bool` | Show full file path |
| `show_checksum` | `bool` | Show MD5 checksum of files |
| `show_stats` | `bool` | Show summary stats at the end |

**Methods:**
- `Config::default()` — creates a Config with sensible defaults (colors on, sort by name, tree format)
- `load_from_file()` — loads config from `~/.config/tree_lens/config.toml` if it exists
- `save_to_file()` — saves current config to `~/.config/tree_lens/config.toml`
- `get_extensions()` — splits `"rs,toml,md"` into `["rs", "toml", "md"]`
- `matches_size_filter(size)` — checks if a file's size is within min/max range

### `src/utils.rs`
Utility/helper functions used across the app.

- `is_hidden(path)` — checks if a file/folder is hidden
- `get_file_extensions(path)` — returns the file extension as lowercase string
- `format_permissions(mode)` — converts unix permission bits to `rwxrwxrwx` string
- `matches_pattern(path, pattern)` — checks if a path matches a regex or glob pattern
- `calculate_md5(path)` — returns the MD5 checksum of a file
- `format_time(timestamp)` — formats a timestamp to `YYYY-MM-DD HH:MM:SS`
- `count_files_in_dir(path)` — returns `(file_count, dir_count)` for a directory
- `parse_size(size_str)` — converts `"10MB"` → `10_000_000` bytes

### `src/lib.rs`
Entry point for the library. Declares `utils` and `config` as public modules.

## Dependencies
| Crate | Purpose |
|---|---|
| `clap` | Parse CLI arguments |
| `serde` | Serialize/Deserialize structs to/from files |
| `toml` | Read and write `.toml` config files |
| `dirs` | Find OS config directory path |
| `walkdir` | Walk directory trees recursively |
| `colored` | Colorful terminal output |
| `regex` | Pattern matching on file paths |
| `chrono` | Format file timestamps |
| `md5` | Calculate file checksums |
| `ignore` | Respect `.gitignore` rules |
| `humansize` | Format bytes as `1.2 MB` etc |
| `git2` | Git integration (optional feature) |

## What's Next
- `main.rs` — wire up CLI with `clap`, connect to `Config`
- Directory walker — use `walkdir` to traverse folders
- Apply filters from `Config` to each file
- Render output in selected `OutputFormat`
- Git status integration
