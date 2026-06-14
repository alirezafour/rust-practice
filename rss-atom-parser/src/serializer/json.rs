use crate::{error::{FeedError, SerializationError}, model::Feed};

use super::FeedSerializer;

pub struct JsonSerializer;

impl FeedSerializer for JsonSerializer {
    fn serialize(&self, feed: &Feed) -> Result<Vec<u8>, FeedError> {
        serde_json::to_vec_pretty(feed)
            .map_err(|e| FeedError::Serialization(SerializationError { message: e.to_string() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_round_trip_json() {
        let feed = Feed {
            title: "T".into(),
            link: "https://x".into(),
            description: None,
            updated: None,
            entries: vec![],
        };
        let bytes = JsonSerializer.serialize(&feed).unwrap();
        let back: Feed = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(feed, back);
    }
}
