use std::path::{Path, PathBuf};
// for ~ instead of /home/user
pub fn format_path(path: &Path, home_dir: &PathBuf) -> String {
    if let Ok(rel) = path.strip_prefix(home_dir) {
        if rel.as_os_str().is_empty() {
            "~".to_string()
        } else {
            format!("~/{}", rel.display())
        }
    } else {
        path.display().to_string()
    }
}
