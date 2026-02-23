use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrVec {
    #[default]
    None,
    Single(String),
    Multiple(Vec<String>),
}

impl StringOrVec {
    pub fn into_vec(self) -> Vec<String> {
        match self {
            Self::None => Vec::new(),
            Self::Single(value) => vec![value],
            Self::Multiple(values) => values,
        }
    }

    pub fn first(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Single(value) => Some(value.as_str()),
            Self::Multiple(values) => values.first().map(|value| value.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StringOrVec;

    #[test]
    fn string_or_vec_helpers_cover_all_shapes() {
        assert_eq!(StringOrVec::None.into_vec(), Vec::<String>::new());
        assert_eq!(StringOrVec::Single("X".into()).into_vec(), vec!["X"]);
        assert_eq!(
            StringOrVec::Multiple(vec!["A".into(), "B".into()]).into_vec(),
            vec!["A", "B"]
        );
        assert_eq!(StringOrVec::Single("A".into()).first(), Some("A"));
        assert_eq!(StringOrVec::Multiple(vec!["A".into()]).first(), Some("A"));
        assert_eq!(StringOrVec::None.first(), None);
    }
}
