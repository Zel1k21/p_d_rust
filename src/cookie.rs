use crate::types::{Request, Response};

impl Request {
    pub fn read_cookie(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|&(k, v)| k == "Cookie" && v.split_once("=").unwrap_or_default().0 == name)
            .map(|item| item.1.split_once("=").unwrap_or_default().1)
    }
}

impl Response {
    pub fn write_cookie(&mut self, name: &str, value: &str, max_age: i32) {
        let header_value = format!("{}={}; Max-Age={}", name, value, max_age);
        self.headers.insert("Set-Cookie".to_string(), header_value);
    }
}
