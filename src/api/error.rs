use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Debug)]
// #[non_exhaustive]
pub enum CoggleError {
    TextTooLong,
    InvalidOrganizationName,
}

impl Error for CoggleError {}

impl Display for CoggleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use CoggleError::{InvalidOrganizationName, TextTooLong};
        match self {
            TextTooLong => write!(f, "Error: the text is too long."),
            InvalidOrganizationName => write!(f, "Error: invalid organization name."),
        }
    }
}
