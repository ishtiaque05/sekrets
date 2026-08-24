use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Screen {
    Locating,
    NoVaultFound { path: PathBuf },
    Locked { path: PathBuf },
}

#[derive(Debug, Clone)]
pub enum Message {
    Located(Option<PathBuf>),
}

#[derive(Default)]
pub struct SekretsApp {
    pub screen: Option<Screen>,
}

impl SekretsApp {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Located(Some(path)) => {
                self.screen = Some(Screen::Locked { path });
            }
            Message::Located(None) => {
                let path = sekrets_core::helpers::directories::get_encrypted_file_path(
                    sekrets_core::encryption::encryptor::ENCRYPTED_FILENAME,
                );
                self.screen = Some(Screen::NoVaultFound { path });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locating_transitions_to_locked_when_vault_found() {
        let mut app = SekretsApp {
            screen: Some(Screen::Locating),
        };
        app.update(Message::Located(Some(PathBuf::from("/tmp/sekrets.enc"))));
        assert!(matches!(app.screen, Some(Screen::Locked { .. })));
    }

    #[test]
    fn locating_transitions_to_no_vault_found_when_absent() {
        let mut app = SekretsApp {
            screen: Some(Screen::Locating),
        };
        app.update(Message::Located(None));
        assert!(matches!(app.screen, Some(Screen::NoVaultFound { .. })));
    }
}
