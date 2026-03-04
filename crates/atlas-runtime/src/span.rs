//! Source location tracking and span utilities

use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

pub type FileId = u32;

/// Represents a location in source code
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Starting byte offset
    pub start: usize,
    /// Ending byte offset (exclusive)
    pub end: usize,
    /// Source file id
    pub file: FileId,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            file: unknown_file_id(),
        }
    }

    /// Create a new span in a specific file
    pub fn new_in(start: usize, end: usize, file: FileId) -> Self {
        Self { start, end, file }
    }

    /// Create a dummy span for testing
    pub fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            file: unknown_file_id(),
        }
    }

    /// Get the source file path for this span
    pub fn file(&self) -> Arc<str> {
        file_path(self.file)
    }

    /// Get the length of this span
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if this span is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this span contains a byte offset
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Check if this span overlaps with another span
    pub fn overlaps(&self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Combine two spans into one encompassing span
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            file: merge_file(self.file, other.file),
        }
    }

    /// Extend this span to include another span
    pub fn extend(&mut self, other: Span) {
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
        self.file = merge_file(self.file, other.file);
    }

    /// Create a span that starts at the end of this span
    pub fn after(&self) -> Span {
        Span {
            start: self.end,
            end: self.end,
            file: self.file,
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Span")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("file", &file_path(self.file).as_ref())
            .finish()
    }
}

impl serde::Serialize for Span {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Span", 3)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.serialize_field("file", &file_path(self.file).as_ref())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Span {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SpanSerde {
            start: usize,
            end: usize,
            file: String,
        }

        let helper = SpanSerde::deserialize(deserializer)?;
        Ok(Span {
            start: helper.start,
            end: helper.end,
            file: intern_file(helper.file),
        })
    }
}

#[derive(Debug)]
struct FileTable {
    ids: HashMap<Arc<str>, FileId>,
    paths: Vec<Arc<str>>,
}

fn file_table() -> &'static Mutex<FileTable> {
    static TABLE: OnceLock<Mutex<FileTable>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let unknown: Arc<str> = Arc::from("<unknown>");
        let mut ids = HashMap::new();
        ids.insert(unknown.clone(), 0);
        let paths = vec![unknown];
        Mutex::new(FileTable { ids, paths })
    })
}

fn unknown_file_id() -> FileId {
    0
}

fn merge_file(left: FileId, right: FileId) -> FileId {
    if left == right {
        return left;
    }
    if left == unknown_file_id() {
        return right;
    }
    if right == unknown_file_id() {
        return left;
    }
    unknown_file_id()
}

pub fn intern_file(path: impl AsRef<str>) -> FileId {
    let path = path.as_ref();
    let mut table = file_table().lock().expect("file table lock poisoned");
    if let Some(id) = table.ids.get(path) {
        return *id;
    }
    let arc: Arc<str> = Arc::from(path);
    let id = table.paths.len() as FileId;
    table.paths.push(arc.clone());
    table.ids.insert(arc, id);
    id
}

pub fn file_path(file: FileId) -> Arc<str> {
    let table = file_table().lock().expect("file table lock poisoned");
    table
        .paths
        .get(file as usize)
        .cloned()
        .unwrap_or_else(|| table.paths[0].clone())
}
