use crate::{error::FeedError, model::Feed, parser::FeedParser};

pub struct AtomParser;

impl FeedParser for AtomParser {
    fn parse(&self, xml: &str) -> Result<Feed, FeedError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
