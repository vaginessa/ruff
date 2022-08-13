use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use walkdir::{DirEntry, WalkDir};

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with('.'))
        .unwrap_or(false)
}

pub fn collect_python_files(path: &PathBuf) -> Vec<DirEntry> {
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(is_not_hidden)
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().to_string_lossy().ends_with(".py"))
        .collect()
}

pub fn readlines(filename: &str, row: &usize) -> String {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut lines: Vec<String> = vec!["\n...".to_string()];
    lines.extend(
        reader
            .lines()
            .skip(*row - 1)
            .take(5)
            .map(|item| item.unwrap())
            .collect::<Vec<String>>(),
    );
    lines.push("...\n".to_string());
    lines.join("\n")
}
