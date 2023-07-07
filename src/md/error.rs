
use std::{fmt::{self, Display}, error::Error};

#[derive(Debug)]
pub enum MdError {}

impl Display for MdError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl Error for MdError {}