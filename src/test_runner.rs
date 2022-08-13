use std::path::PathBuf;

use anyhow::Result;
use log::debug;

use crate::cache;

pub struct Module {
    pub source_root: PathBuf,
    pub path: PathBuf,
}

impl Module {
    fn relative_path(&self) -> PathBuf {
        self.path
            .strip_prefix(&self.source_root)
            .unwrap()
            .to_path_buf()
    }
}

fn run_tests(module: &Module) -> Result<()> {
    // Run unittest.
    let child = std::process::Command::new("/bin/bash")
        .args(&[
            "-c",
            &format!(
                "python -m unittest {}",
                module.relative_path().to_string_lossy()
            ),
        ])
        .current_dir(&module.source_root)
        .output()
        .expect("Failed to run unittest.");
    match child.status.code() {
        Some(0) => Ok(()),
        Some(1) => Err(anyhow::anyhow!(String::from_utf8(child.stderr)?)),
        Some(code) => Err(anyhow::anyhow!("Test runner failed with status {:?}", code)),
        None => Err(anyhow::anyhow!(
            "Unable to determine exit status for test runner."
        )),
    }
}

pub fn check_path(module: &Module, mode: &cache::Mode) -> Result<()> {
    // Check the cache.
    if cache::get(&module.path, mode).is_some() {
        debug!("Cache hit for: {}", module.path.to_string_lossy());
        return Ok(());
    }

    // Run the linter.
    run_tests(module)?;
    cache::set(&module.path, &[], mode);

    Ok(())
}
