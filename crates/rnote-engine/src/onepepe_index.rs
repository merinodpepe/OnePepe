// SPDX-License-Identifier: GPL-3.0-or-later
// OnePepe: the order of the pages and sections of a folder, and which pages are subpages of others, for the tree
// of notebooks. It lives in a hidden file of each folder (`.onepepe-indice.json`): the files themselves never move
// or get renamed. Without it (or if it is deleted) the folder is shown in name order, as before.

// Imports
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::Path;

/// An entry of a folder: a section (subfolder) or a page (or another file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
}

impl Entry {
    pub fn is_page(&self) -> bool {
        !self.is_dir && self.name.ends_with(".rnote")
    }
}

/// The index of a folder.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FolderIndex {
    /// The names in the order chosen by the user. Names not listed go after, in name order.
    #[serde(rename = "orden")]
    pub order: Vec<String>,
    /// Subpage -> its parent page.
    #[serde(rename = "subpaginas")]
    pub parents: BTreeMap<String, String>,
}

impl FolderIndex {
    pub const FILE_NAME: &'static str = ".onepepe-indice.json";

    pub fn load(dir: &Path) -> Self {
        std::fs::read_to_string(dir.join(Self::FILE_NAME))
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    /// Saves the index (or deletes the file when there is nothing to keep).
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        let path = dir.join(Self::FILE_NAME);
        if self.order.is_empty() && self.parents.is_empty() {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            return Ok(());
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// The visible entries of a folder (no hidden files), sorted.
    pub fn read_entries(dir: &Path) -> Vec<Entry> {
        let mut entries: Vec<Entry> = std::fs::read_dir(dir)
            .map(|read| {
                read.filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        (!name.starts_with('.')).then(|| Entry {
                            is_dir: e.file_type().map(|t| t.is_dir()).unwrap_or(false),
                            name,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self::load(dir).sort(&mut entries);
        entries
    }

    /// Sorts the entries: in the user's order; the rest after, sections first, in natural name order.
    pub fn sort(&self, entries: &mut [Entry]) {
        let rank = |e: &Entry| self.order.iter().position(|n| *n == e.name);
        entries.sort_by(|a, b| match (rank(a), rank(b)) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => b
                .is_dir
                .cmp(&a.is_dir)
                .then_with(|| natural_cmp(&a.name, &b.name)),
        });
    }

    /// The parent page of a page, if it still exists in `entries`.
    pub fn parent_of<'a>(&'a self, name: &str, entries: &[Entry]) -> Option<&'a str> {
        let parent = self.parents.get(name)?;
        entries
            .iter()
            .any(|e| e.is_page() && e.name == *parent && e.name != name)
            .then_some(parent.as_str())
    }

    /// The entries directly under `parent` (a page), or at the top of the folder for `None`.
    pub fn children(&self, entries: &[Entry], parent: Option<&str>) -> Vec<Entry> {
        entries
            .iter()
            .filter(|e| self.parent_of(&e.name, entries) == parent)
            .cloned()
            .collect()
    }

    /// Puts `name` just before `before` (or at the end), keeping the current order of the others.
    pub fn move_before(&mut self, entries: &[Entry], name: &str, before: Option<&str>) {
        let mut names: Vec<String> = entries
            .iter()
            .map(|e| e.name.clone())
            .filter(|n| n != name)
            .collect();
        let at = before
            .and_then(|b| names.iter().position(|n| n == b))
            .unwrap_or(names.len());
        names.insert(at, name.to_string());
        self.order = names;
    }

    /// Makes `child` a subpage of `parent` (or a page of its own with `None`). Refuses cycles.
    pub fn set_parent(&mut self, child: &str, parent: Option<&str>) -> bool {
        let Some(parent) = parent else {
            self.parents.remove(child);
            return true;
        };
        // walking up from the new parent must not reach the child
        let mut up = Some(parent.to_string());
        while let Some(p) = up {
            if p == child {
                return false;
            }
            up = self.parents.get(&p).cloned();
        }
        self.parents.insert(child.to_string(), parent.to_string());
        true
    }

    /// OneNote's «Make subpage»: the page becomes a subpage of the page above it at its level.
    pub fn make_subpage(&mut self, entries: &[Entry], name: &str) -> bool {
        let parent = self.parent_of(name, entries).map(str::to_string);
        let siblings = self.children(entries, parent.as_deref());
        let Some(i) = siblings.iter().position(|e| e.name == name) else {
            return false;
        };
        let Some(previous) = i
            .checked_sub(1)
            .map(|i| &siblings[i])
            .filter(|e| e.is_page())
        else {
            return false;
        };
        let previous = previous.name.clone();
        self.set_parent(name, Some(&previous))
    }

    /// OneNote's «Promote subpage»: one level up.
    pub fn promote(&mut self, entries: &[Entry], name: &str) -> bool {
        let Some(parent) = self.parent_of(name, entries).map(str::to_string) else {
            return false;
        };
        let grandparent = self.parent_of(&parent, entries).map(str::to_string);
        self.set_parent(name, grandparent.as_deref())
    }

    /// Follows a rename of a file of the folder.
    pub fn rename(&mut self, old: &str, new: &str) {
        for n in self.order.iter_mut() {
            if n == old {
                *n = new.to_string();
            }
        }
        if let Some(parent) = self.parents.remove(old) {
            self.parents.insert(new.to_string(), parent);
        }
        for parent in self.parents.values_mut() {
            if parent == old {
                *parent = new.to_string();
            }
        }
    }
}

/// Name order where numbers count as numbers ("Tema 2" before "Tema 10"), ignoring case.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut n = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        n.push(c);
                        it.next();
                    }
                    n
                };
                let (na, nb) = (take(&mut a), take(&mut b));
                let ord = na
                    .trim_start_matches('0')
                    .len()
                    .cmp(&nb.trim_start_matches('0').len())
                    .then_with(|| na.trim_start_matches('0').cmp(nb.trim_start_matches('0')));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                let ord = x.to_lowercase().cmp(y.to_lowercase());
                if ord != Ordering::Equal {
                    return ord;
                }
                a.next();
                b.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(name: &str) -> Entry {
        Entry {
            name: name.into(),
            is_dir: false,
        }
    }
    fn section(name: &str) -> Entry {
        Entry {
            name: name.into(),
            is_dir: true,
        }
    }
    fn names(entries: &[Entry]) -> Vec<&str> {
        entries.iter().map(|e| e.name.as_str()).collect()
    }

    #[test]
    fn default_order_is_sections_then_natural_names() {
        let mut entries = vec![
            page("Tema 10.rnote"),
            page("Tema 2.rnote"),
            section("Álgebra"),
        ];
        FolderIndex::default().sort(&mut entries);
        assert_eq!(
            names(&entries),
            ["Álgebra", "Tema 2.rnote", "Tema 10.rnote"]
        );
    }

    #[test]
    fn user_order_and_move() {
        let mut entries = vec![page("a.rnote"), page("b.rnote"), page("c.rnote")];
        let mut index = FolderIndex::default();
        index.move_before(&entries, "c.rnote", Some("a.rnote"));
        index.sort(&mut entries);
        assert_eq!(names(&entries), ["c.rnote", "a.rnote", "b.rnote"]);
        index.move_before(&entries, "c.rnote", None);
        index.sort(&mut entries);
        assert_eq!(names(&entries), ["a.rnote", "b.rnote", "c.rnote"]);
        // a new file goes after the ordered ones
        entries.push(page("0.rnote"));
        index.sort(&mut entries);
        assert_eq!(entries.last().unwrap().name, "0.rnote");
    }

    #[test]
    fn subpages() {
        let entries = vec![page("a.rnote"), page("b.rnote"), page("c.rnote")];
        let mut index = FolderIndex::default();
        assert!(!index.make_subpage(&entries, "a.rnote"));
        assert!(index.make_subpage(&entries, "b.rnote"));
        assert_eq!(
            names(&index.children(&entries, None)),
            ["a.rnote", "c.rnote"]
        );
        assert_eq!(
            names(&index.children(&entries, Some("a.rnote"))),
            ["b.rnote"]
        );
        // c under a, then under b (two levels)
        assert!(index.make_subpage(&entries, "c.rnote"));
        assert_eq!(index.parent_of("c.rnote", &entries), Some("a.rnote"));
        index.move_before(&entries, "c.rnote", None);
        assert!(index.set_parent("c.rnote", Some("b.rnote")));
        assert!(
            !index.set_parent("a.rnote", Some("c.rnote")),
            "cycle refused"
        );
        assert!(index.promote(&entries, "c.rnote"));
        assert_eq!(index.parent_of("c.rnote", &entries), Some("a.rnote"));
        // a deleted parent: the subpage shows at the top
        let without_a = vec![page("b.rnote"), page("c.rnote")];
        assert_eq!(index.children(&without_a, None).len(), 2);
    }

    #[test]
    fn rename_follows_and_files_roundtrip() {
        let entries = vec![page("a.rnote"), page("b.rnote")];
        let mut index = FolderIndex::default();
        index.move_before(&entries, "b.rnote", Some("a.rnote"));
        index.set_parent("b.rnote", Some("a.rnote"));
        index.rename("a.rnote", "x.rnote");
        assert_eq!(index.parents.get("b.rnote").unwrap(), "x.rnote");
        assert_eq!(index.order, ["b.rnote", "x.rnote"]);

        let dir = std::env::temp_dir().join(format!("onepepe-index-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("x.rnote"), b"").unwrap();
        std::fs::write(dir.join("b.rnote"), b"").unwrap();
        index.save(&dir).unwrap();
        assert_eq!(FolderIndex::load(&dir), index);
        assert_eq!(
            names(&FolderIndex::read_entries(&dir)),
            ["b.rnote", "x.rnote"]
        );
        FolderIndex::default().save(&dir).unwrap();
        assert!(!dir.join(FolderIndex::FILE_NAME).exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
