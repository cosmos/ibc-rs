use core::fmt::{Display, Error as FmtError, Formatter};
use tendermint::block::signed_header::SignedHeader;
use tendermint::validator::Set as ValidatorSet;

use alloc::vec::Vec;

pub struct PrettySignedHeader<'a>(pub &'a SignedHeader);

impl Display for PrettySignedHeader<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "SignedHeader {{ header: {{ chain_id: {}, height: {} }}, commit: {{ height: {} }} }}",
            self.0.header.chain_id, self.0.header.height, self.0.commit.height
        )
    }
}

pub struct PrettyValidatorSet<'a>(pub &'a ValidatorSet);

impl Display for PrettyValidatorSet<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let validator_addresses: Vec<_> = self
            .0
            .validators()
            .iter()
            .map(|validator| validator.address)
            .collect();
        if let Some(proposer) = self.0.proposer() {
            match &proposer.name {
                Some(name) => write!(f, "PrettyValidatorSet {{ validators: {}, proposer: {}, total_voting_power: {} }}", PrettySlice(&validator_addresses), name, self.0.total_voting_power()),
                None =>  write!(f, "PrettyValidatorSet {{ validators: {}, proposer: None, total_voting_power: {} }}", PrettySlice(&validator_addresses), self.0.total_voting_power()),
            }
        } else {
            write!(
                f,
                "PrettyValidatorSet {{ validators: {}, proposer: None, total_voting_power: {} }}",
                PrettySlice(&validator_addresses),
                self.0.total_voting_power()
            )
        }
    }
}

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

    use crate::alloc::string::ToString;
    use std::{string::String, vec};

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
