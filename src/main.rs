use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DIRS: &[&str] = &["/Users/richard/proj", "/Users/richard/dev/agora_hedge"];
const EXCLUSIONS: &[&str] = &["target", ".venv", "node_modules"];

fn inspect_dir_for_changes(dir: &Path, last_check: SystemTime) -> std::io::Result<Vec<PathBuf>> {
    let mut changed = Vec::new();

    if !dir.is_dir() {
        println!("{} must be a dir", dir.to_string_lossy());
        std::process::exit(1);
    }

    'parent: for element in fs::read_dir(dir)? {
        let element = element?;
        let element_path = element.path();

        for exc in EXCLUSIONS {
            if element_path.ends_with(exc) {
                continue 'parent;
            }
        }

        if element_path.is_dir() {
            let mut changes = inspect_dir_for_changes(&element_path, last_check)?;
            if changes.is_empty() {
                continue;
            }

            changed.append(&mut changes);
        } else {
            let meta = element_path.metadata()?;
            let modified = meta.modified()?;

            if modified > last_check {
                changed.push(element_path);
            }
        }
    }

    Ok(changed)
}

fn main() {
    loop {
        let last_check = SystemTime::now();
        std::thread::sleep(std::time::Duration::from_secs(10));

        for dir in DIRS {
            println!("Inspecting {dir}...");
            let changes = inspect_dir_for_changes(&PathBuf::from(dir), last_check).unwrap();
            if changes.is_empty() {
                println!("No changes detected");
            } else {
                println!("Changes detected: {:?}", changes);
            }
        }
    }
}
