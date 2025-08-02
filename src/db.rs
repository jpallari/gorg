use crate::fuzzy;
use anyhow::{Context, Result, bail};

pub struct DB {
    data: String,
}

pub struct DBView<'a> {
    lines: Vec<&'a str>,
}

impl DB {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let data = std::fs::read_to_string(path).context("failed to read config")?;
        Ok(Self { data })
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let _ = std::fs::write(path, &self.data)?;
        Ok(())
    }

    pub fn add(&mut self, entry: &str) -> Result<()> {
        let entry = entry.trim();
        if entry.contains(|c: char| c == '\n') {
            bail!("Cannot insert entries that contain new lines: {entry}")
        }

        str_sorted_insert(&mut self.data, entry);
        Ok(())
    }

    pub fn from_entries(entries: &mut [String]) -> Self {
        entries.sort();
        let mut data = String::with_capacity(entries.len() * 100);
        for entry in entries.iter() {
            data.push_str(entry);
            data.push('\n');
        }
        Self { data }
    }

    pub fn find_matches(&self, input: &fuzzy::Keywords) -> impl Iterator<Item = &str> {
        let is_empty = input.is_empty();
        self.data.split('\n').filter_map(move |a| {
            let a = a.trim();
            if a.is_empty() {
                return None
            }
            if is_empty {
                return Some(a)
            }
            match input.score(a) {
                0. => None,
                _ => Some(a),
            }
        })
    }

    pub fn view(&self) -> DBView {
        let lines: Vec<&str> = self.data.split('\n').map(|a| a.trim()).collect();
        DBView { lines }
    }
}

impl<'a> DBView<'a> {
    pub fn find_matches(&self, input: &fuzzy::Keywords, results: &mut Vec<(&'a str, f32)>) {
        results.clear();
        results.extend(self.lines.iter().filter_map(|a| match input.score(a) {
            0. => None,
            score => Some((*a, score)),
        }));
        results.sort_by(|(_, score1), (_, score2)| {
            score2.partial_cmp(score1).expect("Score comparison")
        });
    }
}

fn str_sorted_insert(dest: &mut String, source: &str) {
    let mut count: usize = 0;
    for line in dest.split('\n') {
        if line == source {
            return;
        }
        if line > source {
            break;
        }
        count += line.len() + 1;
    }

    dest.reserve(source.len() + 1);
    if count < dest.len() {
        dest.insert_str(count, source);
        dest.insert_str(count + source.len(), "\n");
    } else {
        dest.push('\n');
        dest.push_str(source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_sorted_insert_start() {
        let mut target = String::from(vec!["aabb", "bbcc", "ccdd"].join("\n"));
        str_sorted_insert(&mut target, "aaab");
        assert_eq!(
            target,
            String::from(vec!["aaab", "aabb", "bbcc", "ccdd",].join("\n"))
        );
    }

    #[test]
    fn str_sorted_insert_middle() {
        let mut target = String::from(vec!["aabb", "bbcc", "ccdd"].join("\n"));
        str_sorted_insert(&mut target, "bbcd");
        assert_eq!(
            target,
            String::from(vec!["aabb", "bbcc", "bbcd", "ccdd",].join("\n"))
        );
    }

    #[test]
    fn str_sorted_insert_end() {
        let mut target = String::from(vec!["aabb", "bbcc", "ccdd"].join("\n"));
        str_sorted_insert(&mut target, "cddd");
        assert_eq!(
            target,
            String::from(vec!["aabb", "bbcc", "ccdd", "cddd",].join("\n"))
        );
    }

    #[test]
    fn str_sorted_insert_dupe() {
        let mut target = String::from(vec!["aabb", "bbcc", "ccdd"].join("\n"));
        str_sorted_insert(&mut target, "bbcc");
        assert_eq!(
            target,
            String::from(vec!["aabb", "bbcc", "ccdd",].join("\n"))
        );
    }
}
