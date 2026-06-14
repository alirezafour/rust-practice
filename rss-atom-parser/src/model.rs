use chrono::{DateTime, Utc};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Feed {
    pub title: String,
    pub link: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub entries: Vec<Entry>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub author: Option<Author>,
    #[serde(default)]
    pub published: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Author {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::chrono::Utc;

    #[test]
    fn feed_round_trip() {
        let feed = Feed {
            title: "My Blog".into(),
            link: "https://example.com".into(),
            description: Some("A blog".into()),
            updated: Some(Utc::now()),
            entries: vec![Entry {
                id: "post-1".into(),
                title: Some("First Post".into()),
                link: Some("https://example.com/1".into()),
                summary: Some("Hello world".into()),
                content: Some("<p>Hello</p>".into()),
                author: Some(Author {
                    name: "Alice".into(),
                    email: Some("alice@example.com".into()),
                    uri: None,
                }),
                published: Some(Utc::now()),
                updated: Some(Utc::now()),
            }],
        };

        let json = serde_json::to_string(&feed).unwrap();
        let back: Feed = serde_json::from_str(&json).unwrap();
        assert_eq!(feed, back);
    }

    #[test]
    fn feed_round_trip_missing_optional() {
        let feed = Feed {
            title: "My Blog".into(),
            link: "https://example.com".into(),
            description: None,
            updated: Some(Utc::now()),
            entries: vec![Entry {
                id: "post-1".into(),
                title: Some("First Post".into()),
                link: Some("https://example.com/1".into()),
                summary: Some("Hello world".into()),
                content: Some("<p>Hello</p>".into()),
                author: Some(Author {
                    name: "Alice".into(),
                    email: Some("alice@example.com".into()),
                    uri: None,
                }),
                published: Some(Utc::now()),
                updated: Some(Utc::now()),
            }],
        };

        let json = serde_json::to_string(&feed).unwrap();
        let back: Feed = serde_json::from_str(&json).unwrap();
        assert_eq!(feed, back);
    }
}
