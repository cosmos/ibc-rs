//! Pretty printing utilities.

use core::fmt::{Display, Error as FmtError, Formatter};

/// A slice type that implements the `Display` trait to pretty-print the contained elements.
pub struct PrettySlice<'a, T>(pub &'a [T]);

impl<'a, T: Display> Display for PrettySlice<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "[ ")?;
        let mut vec_iterator = self.0.iter().peekable();
        while let Some(element) = vec_iterator.next() {
            write!(f, "{element}")?;
            // If it is not the last element, add separator.
            if vec_iterator.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, " ]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_pretty_vec_display() {
        let expected_output = "[ one, two, three ]";

        let string_vec = vec!["one", "two", "three"];
        let pretty_vec = PrettySlice(&string_vec);

        assert_eq!(pretty_vec.to_string(), expected_output);
    }

    #[test]
    fn test_pretty_vec_empty_vec() {
        let expected_output = "[  ]";

        let string_vec: Vec<String> = vec![];
        let pretty_vec = PrettySlice(&string_vec);

        assert_eq!(pretty_vec.to_string(), expected_output);
    }

    #[test]
    fn test_pretty_vec_single_element() {
        let expected_output = "[ one ]";

        let string_vec = vec!["one"];
        let pretty_vec = PrettySlice(&string_vec);

        assert_eq!(pretty_vec.to_string(), expected_output);
    }
}
