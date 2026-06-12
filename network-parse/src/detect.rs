#[derive(Debug, Clone, PartialEq)]
pub enum FeedFormat {
    Rss,
    Atom,
    Unknown,
}

pub fn detect(bytes: &[u8]) -> FeedFormat {
    if let Ok(text) = std::str::from_utf8(bytes) {
        let clean_text = text.trim_start();
        if clean_text.contains("<rss") {
            FeedFormat::Rss
        } else if clean_text.contains("<feed") {
            FeedFormat::Atom
        } else {
            FeedFormat::Unknown
        }
    } else {
        FeedFormat::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_empty() {
        let empty: [u8; 0] = [];
        assert_eq!(detect(&empty), FeedFormat::Unknown);
    }

    #[test]
    fn detect_rss() {
        let xml = b"<?xml version=\"1.0\"?><rss version=\"2.0\"><channel></channel></rss>";
        assert_eq!(detect(xml), FeedFormat::Rss);
    }

    #[test]
    fn detect_atom() {
        let xml = b"<?xml version=\"1.0\"?><feed xmlns=\"http://www.w3.org/2005/Atom\"></feed>";
        assert_eq!(detect(xml), FeedFormat::Atom);
    }

    #[test]
    fn detect_with_whitespace() {
        let xml =
            b"  \n <?xml version=\"1.0\"?>  \n <rss version=\"2.0\"><channel></channel></rss>";
        assert_eq!(detect(xml), FeedFormat::Rss);
    }
}
