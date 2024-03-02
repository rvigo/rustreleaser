use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "version")]
    name: String,
}

impl Tag {
    pub fn new(name: impl Into<String>) -> Self {
        Tag { name: name.into() }
    }

    pub fn value(&self) -> &str {
        &self.name
    }

    /// Strip the leading 'v' from the tag name if it exists
    pub fn strip_v_prefix(&self) -> &str {
        if self.name.starts_with('v') {
            self.name.strip_prefix('v').unwrap_or_default()
        } else {
            &self.name
        }
    }

    pub fn empty() -> Tag {
        Tag {
            name: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_a_new_tag() {
        let tag = Tag::new("v1.0.0");

        assert_eq!(tag.value(), "v1.0.0");
    }

    #[test]
    fn should_strip_v_prefix() {
        let tag = Tag::new("v1.0.0");

        assert_eq!(tag.strip_v_prefix(), "1.0.0");
    }

    #[test]
    fn should_return_the_same_value_when_strip_with_no_v() {
        let tag = Tag::new("1.0.0");

        assert_eq!(tag.strip_v_prefix(), "1.0.0");
    }

    #[test]
    fn should_create_a_empty_tag() {
        let tag = Tag::empty();

        assert_eq!(tag.value(), "");
    }
}
