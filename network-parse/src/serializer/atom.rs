use super::FeedSerializer;
use crate::{
    error::{FeedError, SerializationError},
    model::{Entry, Feed},
};
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

pub struct AtomSerializer;

impl FeedSerializer for AtomSerializer {
    fn serialize(&self, feed: &Feed) -> Result<Vec<u8>, FeedError> {
        let mut w = Writer::new(Vec::new());

        // <feed xmlns="http://www.w3.org/2005/Atom">
        let mut feed_el = BytesStart::new("feed");
        feed_el.push_attribute(("xmlns", "http://www.w3.org/2005/Atom"));
        write_err(w.write_event(Event::Start(feed_el)))?;

        write_text_el(&mut w, "title", &feed.title)?;
        // Atom link is `<link href="..."/>` — self-closing, href attribute.
        // Parser reads it from Event::Empty (atom.rs:41-54), NOT text.
        write_link(&mut w, &feed.link)?;
        if let Some(desc) = &feed.description {
            write_text_el(&mut w, "subtitle", desc)?;
        }
        if let Some(updated) = &feed.updated {
            write_text_el(&mut w, "updated", &updated.to_rfc3339())?;
        }

        for entry in &feed.entries {
            write_entry(&mut w, entry)?;
        }

        write_err(w.write_event(Event::End(BytesEnd::new("feed"))))?;
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

/// Write `<link href="..."/>` — self-closing Empty event.
fn write_link(w: &mut Writer<Vec<u8>>, href: &str) -> Result<(), FeedError> {
    let mut el = BytesStart::new("link");
    el.push_attribute(("href", href));
    write_err(w.write_event(Event::Empty(el)))
}

fn write_entry(w: &mut Writer<Vec<u8>>, e: &Entry) -> Result<(), FeedError> {
    write_err(w.write_event(Event::Start(BytesStart::new("entry"))))?;
    // Round-trip contract: parser reads <id> as entry identity.
    write_text_el(w, "id", &e.id)?;
    if let Some(title) = &e.title {
        write_text_el(w, "title", title)?;
    }
    if let Some(link) = &e.link {
        write_link(w, link)?;
    }
    if let Some(summary) = &e.summary {
        write_text_el(w, "summary", summary)?;
    }
    if let Some(published) = &e.published {
        write_text_el(w, "published", &published.to_rfc3339())?;
    }
    if let Some(updated) = &e.updated {
        write_text_el(w, "updated", &updated.to_rfc3339())?;
    }
    if let Some(author) = &e.author {
        // Parser reads only <name> (atom.rs:184-192). email/uri lost on round-trip.
        write_err(w.write_event(Event::Start(BytesStart::new("author"))))?;
        write_text_el(w, "name", &author.name)?;
        write_err(w.write_event(Event::End(BytesEnd::new("author"))))?;
    }
    write_err(w.write_event(Event::End(BytesEnd::new("entry"))))?;
    Ok(())
}

fn write_err(r: Result<(), std::io::Error>) -> Result<(), FeedError> {
    r.map_err(|e| FeedError::Serialization(SerializationError { message: e.to_string() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Author;
    use crate::parser::{FeedParser, atom::AtomParser};
    use chrono::TimeZone;
    use chrono::Utc;

    fn sample_feed() -> Feed {
        Feed {
            title: "My Blog".into(),
            link: "https://example.com".into(),
            description: Some("A blog".into()),
            updated: Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap()),
            entries: vec![Entry {
                id: "post-1".into(),
                title: Some("First & Second <third>".into()),
                link: Some("https://example.com/1".into()),
                summary: Some("Tom & Jerry <cartoon>".into()),
                content: None,
                author: Some(Author {
                    name: "Alice".into(),
                    email: None,
                    uri: None,
                }),
                published: Some(Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap()),
                updated: Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap()),
            }],
        }
    }

    #[test]
    fn atom_serialize_then_parse_round_trips() {
        let feed = sample_feed();
        let xml_bytes = AtomSerializer.serialize(&feed).unwrap();
        let xml = String::from_utf8(xml_bytes).unwrap();
        let back = AtomParser.parse(&xml).unwrap();

        assert_eq!(back.title, feed.title);
        assert_eq!(back.link, feed.link);
        assert_eq!(back.description, feed.description);
        assert_eq!(back.updated, feed.updated);
        assert_eq!(back.entries.len(), 1);
        assert_eq!(back.entries[0].id, feed.entries[0].id);
        assert_eq!(back.entries[0].title, feed.entries[0].title);
        assert_eq!(back.entries[0].link, feed.entries[0].link);
        assert_eq!(back.entries[0].summary, feed.entries[0].summary);
        assert_eq!(back.entries[0].published, feed.entries[0].published);
        assert_eq!(back.entries[0].updated, feed.entries[0].updated);
        assert_eq!(
            back.entries[0].author.as_ref().map(|a| &a.name),
            feed.entries[0].author.as_ref().map(|a| &a.name)
        );
    }

    #[test]
    fn atom_serialize_escapes_special_chars() {
        let feed = sample_feed();
        let xml = String::from_utf8(AtomSerializer.serialize(&feed).unwrap()).unwrap();
        assert!(xml.contains("First &amp; Second &lt;third&gt;"));
        assert!(!xml.contains("First & Second"));
    }

    #[test]
    fn atom_serialize_empty_feed() {
        let feed = Feed {
            title: "Empty".into(),
            link: "https://e".into(),
            description: None,
            updated: None,
            entries: vec![],
        };
        let xml = String::from_utf8(AtomSerializer.serialize(&feed).unwrap()).unwrap();
        let back = AtomParser.parse(&xml).unwrap();
        assert_eq!(back.title, "Empty");
        assert!(back.entries.is_empty());
        assert_eq!(back.description, None);
    }

    #[test]
    fn atom_serialize_emits_self_closing_link() {
        let feed = sample_feed();
        let xml = String::from_utf8(AtomSerializer.serialize(&feed).unwrap()).unwrap();
        // Parser reads link from Event::Empty with href attr — must be self-closing.
        assert!(xml.contains("<link href=\"https://example.com\"/>"));
        assert!(xml.contains("<link href=\"https://example.com/1\"/>"));
    }
}
