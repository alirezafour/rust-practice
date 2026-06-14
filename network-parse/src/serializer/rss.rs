use super::FeedSerializer;
use crate::{
    error::{FeedError, SerializationError},
    model::{Entry, Feed},
};
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

pub struct RssSerializer;

impl FeedSerializer for RssSerializer {
    fn serialize(&self, feed: &Feed) -> Result<Vec<u8>, FeedError> {
        let mut w = Writer::new(Vec::new());

        // <rss version="2.0">
        let mut rss = BytesStart::new("rss");
        rss.push_attribute(("version", "2.0"));
        write_err(w.write_event(Event::Start(rss)))?;

        // <channel>
        write_err(w.write_event(Event::Start(BytesStart::new("channel"))))?;
        write_text_el(&mut w, "title", &feed.title)?;
        write_text_el(&mut w, "link", &feed.link)?;
        if let Some(desc) = &feed.description {
            write_text_el(&mut w, "description", desc)?;
        }

        for entry in &feed.entries {
            write_item(&mut w, entry)?;
        }

        // </channel></rss>
        write_err(w.write_event(Event::End(BytesEnd::new("channel"))))?;
        write_err(w.write_event(Event::End(BytesEnd::new("rss"))))?;

        Ok(w.into_inner())
    }
}

/// Write `<tag>escaped text</tag>`.
fn write_text_el(w: &mut Writer<Vec<u8>>, tag: &str, text: &str) -> Result<(), FeedError> {
    write_err(w.write_event(Event::Start(BytesStart::new(tag))))?;
    write_err(w.write_event(Event::Text(BytesText::new(text))))?;
    write_err(w.write_event(Event::End(BytesEnd::new(tag))))?;
    Ok(())
}

fn write_item(w: &mut Writer<Vec<u8>>, e: &Entry) -> Result<(), FeedError> {
    write_err(w.write_event(Event::Start(BytesStart::new("item"))))?;
    // Round-trip contract: parser sets id = link, so emit link before any id.
    // RSS has no standalone <id>; we reuse <link> as identity.
    if let Some(link) = &e.link {
        write_text_el(w, "link", link)?;
    }
    if let Some(title) = &e.title {
        write_text_el(w, "title", title)?;
    }
    if let Some(summary) = &e.summary {
        write_text_el(w, "description", summary)?;
    }
    if let Some(published) = &e.published {
        // RFC 2822 — matches RssParser::parse_rfc2822 on the way back in.
        write_text_el(w, "pubDate", &published.to_rfc2822())?;
    }
    write_err(w.write_event(Event::End(BytesEnd::new("item"))))?;
    Ok(())
}

fn write_err(r: Result<(), std::io::Error>) -> Result<(), FeedError> {
    r.map_err(|e| FeedError::Serialization(SerializationError { message: e.to_string() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{FeedParser, rss::RssParser};
    use chrono::TimeZone;

    fn sample_feed() -> Feed {
        Feed {
            title: "My Blog".into(),
            link: "https://example.com".into(),
            description: Some("A blog".into()),
            updated: None,
            entries: vec![Entry {
                id: "https://example.com/1".into(),
                title: Some("First Post".into()),
                link: Some("https://example.com/1".into()),
                summary: Some("Hello & goodbye <world>".into()),
                content: None,
                author: None,
                published: Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap()),
                updated: None,
            }],
        }
    }

    // Local alias so the helper struct literal above reads cleanly.
    use chrono::Utc;

    #[test]
    fn rss_serialize_then_parse_round_trips() {
        let feed = sample_feed();
        let xml_bytes = RssSerializer.serialize(&feed).unwrap();
        let xml = String::from_utf8(xml_bytes).unwrap();
        let back = RssParser.parse(&xml).unwrap();

        assert_eq!(back.title, feed.title);
        assert_eq!(back.link, feed.link);
        assert_eq!(back.description, feed.description);
        assert_eq!(back.entries.len(), 1);
        assert_eq!(back.entries[0].title, feed.entries[0].title);
        assert_eq!(back.entries[0].link, feed.entries[0].link);
        assert_eq!(back.entries[0].summary, feed.entries[0].summary);
        assert_eq!(back.entries[0].published, feed.entries[0].published);
    }

    #[test]
    fn rss_serialize_escapes_special_chars() {
        let feed = sample_feed();
        let xml = String::from_utf8(RssSerializer.serialize(&feed).unwrap()).unwrap();
        // Serializer must escape, not emit raw special chars inside text.
        assert!(xml.contains("Hello &amp; goodbye &lt;world&gt;"));
        assert!(!xml.contains("Hello & goodbye"));
    }

    #[test]
    fn rss_serialize_empty_feed() {
        let feed = Feed {
            title: "Empty".into(),
            link: "https://e".into(),
            description: None,
            updated: None,
            entries: vec![],
        };
        let xml = String::from_utf8(RssSerializer.serialize(&feed).unwrap()).unwrap();
        let back = RssParser.parse(&xml).unwrap();
        assert_eq!(back.title, "Empty");
        assert!(back.entries.is_empty());
        assert_eq!(back.description, None);
    }
}
