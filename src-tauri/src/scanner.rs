use std::path::Path;
use std::fs;

pub fn scan_directories(folder: &str) -> Vec<String> {
    let mut images = Vec::new();
    let extensions = ["jpg", "jpeg", "png", "bmp", "webp"];

    let path = Path::new(folder);
    if !path.exists() || !path.is_dir() {
        return images;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if extensions.contains(&ext.to_lowercase().as_str()) {
                        if let Some(path_str) = p.to_str() {
                            images.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }
    images
}
