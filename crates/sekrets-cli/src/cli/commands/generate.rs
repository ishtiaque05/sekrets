use crate::interactive::interactive_mode;
use anyhow::Result;
use sekrets_core::secrets::password_generator::PasswordGenerationError;

pub fn generate_strong_password(flag: bool) -> Result<()> {
    if flag {
        interactive_mode().map_err(anyhow::Error::from)?;
        Ok(())
    } else {
        Err(PasswordGenerationError::NoChoiceSelected.into())
    }
}
