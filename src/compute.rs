pub trait Compute {
    fn get_element_ref(&mut self) -> Option<scraper::ElementRef>;
    fn to_url(&mut self, base_url: &'static str) -> Option<String> {
        self
            .get_element_ref()
            .map(|e| format!("{}{}", base_url, e.value().attr("href").unwrap()))
    }
    fn to_text(&mut self) -> Option<&str> {
        self
            .get_element_ref()
            .and_then(|e| e.text().next())
            .map(|s| s.trim())
    }
}

impl<'a, 'b> Compute for scraper::element_ref::Select<'a, 'b> {
     fn get_element_ref(&mut self) -> Option<scraper::ElementRef> {
        self.next()
    }
}

impl<'a, 'b> Compute for scraper::html::Select<'a, 'b> {
    fn get_element_ref(&mut self) -> Option<scraper::ElementRef> {
        self.next()
    }
}

pub trait ComputeCount {
    fn get_text_internal(&self, count: usize) -> Option<&str>;
    fn get_text(&self) -> Option<&str> {
        self.get_text_internal(0).map(|s| s.trim())
    }
    fn get_text_count(&self, count: usize) -> Option<&str> {
        self.get_text_internal(count).map(|s| s.trim())
    }
    fn get_string(&self) -> Option<String> {
        self.get_text().map(|s| s.to_string())
    }
    fn get_string_count(&self, count: usize) -> Option<String> {
        self.get_text_count(count).map(|s| s.to_string())
    }
}

impl<T: ComputeCount> ComputeCount for Option<T> {
    fn get_text_internal(&self, count: usize) -> Option<&str> {
        self.as_ref().and_then(|c| c.get_text_internal(count))
    }
}

impl<'a> ComputeCount for regex::Captures<'a> {
    fn get_text_internal(&self, count: usize) -> Option<&str> {
        self.get(count).map(|m| m.as_str())
    }
}

impl<'a> ComputeCount for scraper::element_ref::ElementRef<'a> {
    fn get_text_internal(&self, count: usize) -> Option<&str> {
        self.text().nth(count)
    }
}
