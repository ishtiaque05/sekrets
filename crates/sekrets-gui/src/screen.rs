use std::path::PathBuf;
use std::time::Instant;

use iced::widget::{button, column, text, text_input};
use iced::{Element, Task};
use sekrets_core::{Vault, VaultError};

pub enum Screen {
    Locating,
    NoVaultFound {
        path: PathBuf,
    },
    // `path` is retained on `Locked` for future screens (e.g. displaying which
    // vault file is being unlocked); not yet read by this task's `view()`.
    #[allow(dead_code)]
    Locked {
        path: PathBuf,
        password: String,
        error: Option<VaultError>,
        unlocking: bool,
    },
    // Fields consumed by Task D5 (migration) and Phase E (list view), not yet
    // read by this task.
    #[allow(dead_code)]
    Unlocked {
        vault: Vault,
        view: UnlockedView,
        last_activity: Instant,
    },
}

#[allow(dead_code)]
pub enum UnlockedView {
    List { query: String },
}

#[derive(Debug, Clone)]
pub enum Message {
    Located(Option<PathBuf>),
    PasswordChanged(String),
    UnlockSubmitted,
    UnlockCompleted(Result<Vault, VaultError>),
}

impl std::fmt::Debug for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Screen")
    }
}

#[derive(Default)]
pub struct SekretsApp {
    pub screen: Option<Screen>,
}

impl SekretsApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Located(Some(path)) => {
                self.screen = Some(Screen::Locked {
                    path,
                    password: String::new(),
                    error: None,
                    unlocking: false,
                });
                Task::none()
            }
            Message::Located(None) => {
                let path = sekrets_core::helpers::directories::get_encrypted_file_path(
                    sekrets_core::encryption::encryptor::ENCRYPTED_FILENAME,
                );
                self.screen = Some(Screen::NoVaultFound { path });
                Task::none()
            }
            Message::PasswordChanged(new_password) => {
                if let Some(Screen::Locked { password, .. }) = &mut self.screen {
                    *password = new_password;
                }
                Task::none()
            }
            Message::UnlockSubmitted => {
                let password = match &mut self.screen {
                    Some(Screen::Locked {
                        password,
                        unlocking,
                        ..
                    }) => {
                        *unlocking = true;
                        password.clone()
                    }
                    _ => return Task::none(),
                };
                Task::perform(
                    async move { Vault::unlock(&password) },
                    Message::UnlockCompleted,
                )
            }
            Message::UnlockCompleted(Ok(vault)) => {
                self.screen = Some(Screen::Unlocked {
                    vault,
                    view: UnlockedView::List {
                        query: String::new(),
                    },
                    last_activity: Instant::now(),
                });
                Task::none()
            }
            Message::UnlockCompleted(Err(err)) => {
                if let Some(Screen::Locked {
                    error, unlocking, ..
                }) = &mut self.screen
                {
                    *error = Some(err);
                    *unlocking = false;
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Some(Screen::Locating) => text("Looking for your sekrets file...").into(),
            Some(Screen::NoVaultFound { path }) => {
                text(format!("No sekrets file found at {}", path.display())).into()
            }
            Some(Screen::Locked {
                password,
                error,
                unlocking,
                ..
            }) => {
                let mut col = column![
                    text("Enter your master password"),
                    text_input("Master password", password)
                        .on_input(Message::PasswordChanged)
                        .on_submit(Message::UnlockSubmitted)
                        .secure(true),
                    button(if *unlocking { "Unlocking..." } else { "Unlock" })
                        .on_press(Message::UnlockSubmitted),
                ]
                .spacing(10);
                if let Some(err) = error {
                    col = col.push(text(err.to_string()));
                }
                col.into()
            }
            Some(Screen::Unlocked { .. }) => text("Unlocked").into(),
            None => text("").into(),
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
        let _ = app.update(Message::Located(Some(PathBuf::from("/tmp/sekrets.enc"))));
        assert!(matches!(app.screen, Some(Screen::Locked { .. })));
    }

    #[test]
    fn locating_transitions_to_no_vault_found_when_absent() {
        let mut app = SekretsApp {
            screen: Some(Screen::Locating),
        };
        let _ = app.update(Message::Located(None));
        assert!(matches!(app.screen, Some(Screen::NoVaultFound { .. })));
    }

    #[test]
    fn password_input_updates_locked_screen_field() {
        let mut app = SekretsApp {
            screen: Some(Screen::Locked {
                path: PathBuf::from("/tmp/sekrets.enc"),
                password: String::new(),
                error: None,
                unlocking: false,
            }),
        };
        let _ = app.update(Message::PasswordChanged("hunter2".to_string()));
        match &app.screen {
            Some(Screen::Locked { password, .. }) => assert_eq!(password, "hunter2"),
            _ => panic!("expected Locked screen"),
        }
    }

    #[test]
    fn unlock_completed_success_transitions_to_unlocked() {
        let mut app = SekretsApp {
            screen: Some(Screen::Locked {
                path: PathBuf::from("/tmp/sekrets.enc"),
                password: "hunter2".to_string(),
                error: None,
                unlocking: true,
            }),
        };
        // Vault has no public constructor for tests outside sekrets-core; use a real
        // Vault::create/unlock roundtrip under test-utils isolation instead of a mock.
        sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let vault = sekrets_core::Vault::unlock("hunter2").expect("unlock should succeed");

        let _ = app.update(Message::UnlockCompleted(Ok(vault)));
        assert!(matches!(app.screen, Some(Screen::Unlocked { .. })));
    }

    #[test]
    fn unlock_completed_failure_stays_locked_with_error() {
        let mut app = SekretsApp {
            screen: Some(Screen::Locked {
                path: PathBuf::from("/tmp/sekrets.enc"),
                password: "wrong".to_string(),
                error: None,
                unlocking: true,
            }),
        };
        let _ = app.update(Message::UnlockCompleted(Err(
            sekrets_core::VaultError::WrongPassword,
        )));
        match &app.screen {
            Some(Screen::Locked {
                error, unlocking, ..
            }) => {
                assert!(matches!(
                    error,
                    Some(sekrets_core::VaultError::WrongPassword)
                ));
                assert!(!unlocking);
            }
            _ => panic!("expected Locked screen"),
        }
    }
}
