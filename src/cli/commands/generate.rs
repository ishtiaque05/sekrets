use crate::secrets::password_generator::{PasswordGenerationError, PasswordGenerator};
use anyhow::Result;

pub fn generate_strong_password(flag: bool) -> Result<()> {
    if flag {
        PasswordGenerator::interactive_mode().map_err(anyhow::Error::from)?;
        Ok(())
    } else {
        Err(PasswordGenerationError::NoChoiceSelected.into())
    }
}
