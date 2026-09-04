//! Why a navigation request was refused.

/// Why a navigation request was refused.
///
/// Returned by the `Result`-returning methods of
/// [`Router`](crate::Router). A navigation that arrives through
/// [`Router::update`](crate::Router::update) instead is a programming error
/// and is logged (and asserted in debug builds) rather than returned.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NavigationError {
    /// The page type was never added to this router.
    UnknownPage {
        /// [`std::any::type_name`] of the page. Used only for diagnostics; its
        /// exact text is not stable.
        type_name: &'static str,
    },
    /// The index is past the end of the page list.
    IndexOutOfRange {
        /// The requested index.
        index: usize,
        /// How many pages the router has.
        len: usize,
    },
}

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPage { type_name } => {
                write!(f, "page {type_name} is not in this router")
            }
            Self::IndexOutOfRange { index, len } => {
                write!(
                    f,
                    "page index {index} is out of range (the router has {len} pages)"
                )
            }
        }
    }
}

impl std::error::Error for NavigationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_the_page_or_index() {
        let unknown = NavigationError::UnknownPage {
            type_name: "app::Home",
        };
        assert_eq!(unknown.to_string(), "page app::Home is not in this router");
        let range = NavigationError::IndexOutOfRange { index: 9, len: 2 };
        assert_eq!(
            range.to_string(),
            "page index 9 is out of range (the router has 2 pages)"
        );
        let boxed: Box<dyn std::error::Error> = Box::new(range);
        assert!(boxed.source().is_none());
    }
}
