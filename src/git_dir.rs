use std::path::{Path, PathBuf};

pub struct GitDirIterator {
    search_stack: Vec<PathBuf>,
}

impl GitDirIterator {
    pub fn new<P: AsRef<Path>>(start_dir: P) -> Self {
        let start_dir = start_dir.as_ref();
        if !start_dir.is_dir() {
            panic!("Given path is not a directory");
        }
        Self {
            search_stack: vec![start_dir.to_path_buf()],
        }
    }
}

impl Iterator for GitDirIterator {
    type Item = std::io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        let git_os_str = std::ffi::OsStr::new(".git");
        loop {
            let Some(next_dir) = self.search_stack.pop() else {
                return None;
            };

            let entries = match std::fs::read_dir(&next_dir) {
                Ok(entries) => entries,
                Err(err) => {
                    return Some(Err(err));
                }
            };

            let mut pushed_items = 0;
            let mut result = None;
            'entry: for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        result = Some(Err(err));
                        break 'entry;
                    }
                };
                let path = entry.path();
                if path.is_dir() && path.file_name() == Some(git_os_str) {
                    result = Some(Ok(next_dir));
                    break 'entry;
                }
                if path.is_dir() {
                    pushed_items += 1;
                    self.search_stack.push(path);
                }
            }

            if result.is_some() {
                if pushed_items > 0 {
                    self.search_stack
                        .truncate(self.search_stack.len() - pushed_items);
                }
                return result;
            }
        }
    }
}
