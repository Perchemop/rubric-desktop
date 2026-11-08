//! A threaded comment attached to a markup.
//!
//! Comments live on [`crate::domain::Markup`] and form a flat per-markup list.
//! Each comment carries its own id, author, text and creation timestamp.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single comment on a markup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub author: String,
    pub text: String,
    pub created: DateTime<Utc>,
}

impl Comment {
    /// Create a new comment with a fresh id and the current timestamp.
    pub fn new(author: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            author: author.into(),
            text: text.into(),
            created: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_new_sets_fields() {
        let c = Comment::new("Ann", "looks good");
        assert_eq!(c.author, "Ann");
        assert_eq!(c.text, "looks good");
        assert!(c.created <= Utc::now());
    }

    #[test]
    fn comment_ids_are_unique() {
        let a = Comment::new("Ann", "x");
        let b = Comment::new("Ann", "x");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn comment_serde_roundtrip() {
        let c = Comment::new("Ann", "looks good");
        let json = serde_json::to_string(&c).expect("serialize");
        let back: Comment = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(c, back);
    }

    #[test]
    fn comment_partial_eq_ignores_id_order() {
        let mut a = Comment::new("Ann", "looks good");
        let mut b = Comment::new("Ann", "looks good");
        b.id = a.id;
        b.created = a.created;
        assert_eq!(a, b);
    }
}
