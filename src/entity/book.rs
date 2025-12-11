use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub pages: Vec<String>,
    #[serde(default = "BookEntry::default_writable")]
    pub writable: bool,
}

impl BookEntry {
    fn default_writable() -> bool {
        true
    }

    pub fn new(id: String, title: impl Into<String>, writable: bool) -> Self {
        Self {
            id,
            title: title.into(),
            pages: Vec::new(),
            writable,
        }
    }

    pub fn set_page(&mut self, page_index: usize, content: impl Into<String>) {
        let idx = page_index;
        if self.pages.len() <= idx {
            self.pages.resize(idx + 1, String::new());
        }
        self.pages[idx] = content.into();
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn summary(&self) -> String {
        let page_info = if self.pages.is_empty() {
            "no pages yet".to_string()
        } else {
            format!("{} page(s)", self.pages.len())
        };
        format!("Book [{}]: {} ({})", self.id, self.title, page_info)
    }

    pub fn full_text(&self) -> String {
        if self.pages.is_empty() {
            return format!("Book [{}]: {}\n(no pages written)", self.id, self.title);
        }
        let mut out = format!("Book [{}]: {}\n", self.id, self.title);
        for (i, page) in self.pages.iter().enumerate() {
            out.push_str(&format!("Page {}: {}\n", i + 1, page));
        }
        out.trim_end().to_string()
    }
}
