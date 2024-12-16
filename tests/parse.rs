use gall_rs::parse::{get_method, get_path, internal_parse};
use gall_rs::types::{HttpParseError, HttpVersion, Method, Request};

#[cfg(test)]
mod test_parse {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn it_gets_method() {
        assert_eq!(get_method(None), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some(" ")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("get")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("pUt")), Err(HttpParseError::InvalidMethod));
        assert_eq!(get_method(Some("rand")), Err(HttpParseError::InvalidMethod));

        assert_eq!(get_method(Some("GET")), Ok(Method::Get));
        assert_eq!(get_method(Some("POST")), Ok(Method::Post));
        assert_eq!(get_method(Some("PATCH")), Ok(Method::Patch));
        assert_eq!(get_method(Some("DELETE")), Ok(Method::Delete));
        assert_eq!(get_method(Some("PUT")), Ok(Method::Put));
        assert_eq!(get_method(Some("OPTIONS")), Ok(Method::Options));
        assert_eq!(get_method(Some("HEAD")), Ok(Method::Head));
    }

    #[test]
    fn it_gets_path() {
        assert_eq!(get_path(None), Err(HttpParseError::InvalidPath));
        assert_eq!(get_path(Some("")), Err(HttpParseError::InvalidPath));

        assert_eq!(get_path(Some("/a/path")), Ok("/a/path".to_owned()));
    }

    #[test]
    fn it_constructs_request() {
        assert_eq!(
            internal_parse(&"".as_bytes().to_vec()),
            Err(HttpParseError::InvalidMethod)
        );
        assert_eq!(
            internal_parse(&"g".as_bytes().to_vec()),
            Err(HttpParseError::InvalidMethod)
        );

        assert_eq!(
            internal_parse(&"GET".as_bytes().to_vec()),
            Err(HttpParseError::InvalidPath)
        );
        assert_eq!(
            internal_parse(&"GET   ".as_bytes().to_vec()),
            Err(HttpParseError::InvalidPath)
        );
        assert_eq!(
            internal_parse(&"GET /path".as_bytes().to_vec()),
            Err(HttpParseError::InvalidHttpVersion)
        );
        assert_eq!(
            internal_parse(&"GET /path HTTP/1.0".as_bytes().to_vec()),
            Err(HttpParseError::InvalidHttpVersion)
        );

        assert_eq!(
            internal_parse(&"GET /path HTTP/1.1".as_bytes().to_vec()),
            Ok(Request {
                method: Method::Get,
                path: "/path".to_owned(),
                http_version: HttpVersion::Http1_1,
                headers: HashMap::new(),
                body: None,
            })
        )
    }
}
