use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total_elements: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> Page<T> {
    pub fn new(content: Vec<T>, page: u32, page_size: u32, total_elements: u64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            ((total_elements + page_size as u64 - 1) / page_size as u64) as u32
        };
        let has_next = page < total_pages;
        let has_previous = page > 1 && total_pages > 0;

        Self {
            content,
            page,
            page_size,
            total_elements,
            total_pages,
            has_next,
            has_previous,
        }
    }

    pub fn empty(page_request: &PageRequest) -> Self {
        Self::new(Vec::new(), page_request.page, page_request.page_size, 0)
    }

    pub fn map<U, F>(self, f: F) -> Page<U>
    where
        F: FnMut(T) -> U,
    {
        Page {
            content: self.content.into_iter().map(f).collect(),
            page: self.page,
            page_size: self.page_size,
            total_elements: self.total_elements,
            total_pages: self.total_pages,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRequest {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}

impl PageRequest {
    pub fn new(page: u32, page_size: u32) -> Self {
        Self { page, page_size }
    }

    pub fn offset(&self) -> u32 {
        if self.page == 0 {
            0
        } else {
            (self.page - 1) * self.page_size
        }
    }

    pub fn limit(&self) -> u32 {
        self.page_size
    }
}

pub type PaginationRequest = PageRequest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_calculations() {
        let items = vec![1, 2, 3];
        let page = Page::new(items.clone(), 1, 3, 10);
        assert_eq!(page.total_pages, 4);
        assert!(page.has_next);
        assert!(!page.has_previous);

        let page2 = Page::new(items.clone(), 2, 3, 10);
        assert!(page2.has_next);
        assert!(page2.has_previous);

        let page4 = Page::new(vec![10], 4, 3, 10);
        assert!(!page4.has_next);
        assert!(page4.has_previous);
    }

    #[test]
    fn test_page_map() {
        let page = Page::new(vec![1, 2, 3], 1, 10, 3);
        let mapped = page.map(|n| n.to_string());
        assert_eq!(mapped.content, vec!["1", "2", "3"]);
        assert_eq!(mapped.total_elements, 3);
    }
}
