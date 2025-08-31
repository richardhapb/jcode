use std::error::Error;
use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DIRS: &[&str] = &["/Users/richard/proj", "/Users/richard/dev"];
const EXCLUSIONS: &[&str] = &["target", ".venv", "node_modules", ".git", ".DS_Store"];

#[derive(Debug)]
struct JCodeError(String);

impl From<std::io::Error> for JCodeError {
    fn from(value: std::io::Error) -> Self {
        Self(format!("{value}"))
    }
}

impl Display for JCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for JCodeError {}

#[derive(Debug)]
struct Change {
    path: PathBuf,
    timestamp: SystemTime,
}

impl Change {
    fn new(path: PathBuf, timestamp: SystemTime) -> Self {
        Self { path, timestamp }
    }
}

trait DataHandler {
    fn save(&self, changes: Vec<Change>) -> Result<(), JCodeError>;
}

struct CsvHandler {
    path: PathBuf,
}

impl CsvHandler {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl DataHandler for CsvHandler {
    fn save(&self, changes: Vec<Change>) -> Result<(), JCodeError> {
        let exists = self.path.exists();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        if !exists {
            file.write_all(b"path,timestamp\n")?;
        }

        for c in changes {
            // Always end with newline
            let line = format!(
                "\"{}\",{}\n",
                c.path.to_string_lossy().replace('"', "\"\""),
                c.timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
            file.write_all(line.as_bytes())?;
        }
        Ok(())
    }
}

fn inspect_dir_for_changes(dir: &Path, last_check: SystemTime) -> std::io::Result<Vec<Change>> {
    let mut changed = Vec::new();

    if !dir.is_dir() {
        eprintln!("{} must be a dir", dir.to_string_lossy());
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
                changed.push(Change::new(element_path, modified));
            }
        }
    }

    Ok(changed)
}

fn main() {
    let file_path = "changes.csv";
    let csvh = CsvHandler::new(file_path.into());

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
                csvh.save(changes).unwrap();
                println!("Saved to {}", file_path);
            }
        }
    }
}
