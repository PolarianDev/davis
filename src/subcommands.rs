use std::collections::HashMap;
use std::env;
use std::ffi;
use std::fs;
use std::iter::once;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn find_subcommands() -> HashMap<String, PathBuf> {
    let path_str = env::var("PATH").expect("$PATH was not set!");
    let home = env::var("HOME").expect("$HOME was not set!");
    let home_subcommands: PathBuf = [&*home, ".config", "davis", "bin"].iter().collect();
    let etc_subcommands: PathBuf = ["/", "etc", "davis", "bin"].iter().collect();
    let custom_dirs = once(home_subcommands).chain(once(etc_subcommands.clone()));
    let paths = env::split_paths(&*path_str).chain(custom_dirs);

    paths
        .flat_map(|p| fs::read_dir(p).into_iter().flatten())
        .flat_map(|d| d.into_iter())
        .filter_map(|d| {
            let file_name = d.file_name().to_string_lossy().to_string();
            if file_name.starts_with("davis-") && is_executable(&d) {
                Some((file_name, d.path()))
            } else {
                None
            }
        })
        .collect()
}

// copied from https://github.com/frewsxcv/rust-quale
fn is_executable(file: &fs::DirEntry) -> bool {
    // Don't use `file.metadata()` directly since it doesn't follow symlinks.
    let file_metadata = match file.path().metadata() {
        Ok(metadata) => metadata,
        Err(..) => return false,
    };
    let file_path = match file.path().to_str().and_then(|p| ffi::CString::new(p).ok()) {
        Some(path) => path,
        None => return false,
    };
    let is_executable_by_user =
        unsafe { libc::access(file_path.into_raw(), libc::X_OK) == libc::EXIT_SUCCESS };
    static EXECUTABLE_FLAGS: u32 = (libc::S_IEXEC | libc::S_IXGRP | libc::S_IXOTH) as u32;
    let has_executable_flag = file_metadata.permissions().mode() & EXECUTABLE_FLAGS != 0;
    is_executable_by_user && has_executable_flag && file_metadata.is_file()
}