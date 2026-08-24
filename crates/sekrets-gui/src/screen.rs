use std::path::PathBuf;
use std::time::Instant;

use iced::widget::{button, column, text, text_input};
use iced::{Element, Task};
use sekrets_core::{Vault, VaultError};

pub enum Screen {
    Locating,
    NoVaultFound {
        path: PathBuf,
        password: String,
        confirm: String,
        error: Option<String>,
        creating: bool,
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
    CreatePasswordChanged(String),
    CreateConfirmChanged(String),
    CreateSubmitted,
    CreateCompleted(Result<Vault, VaultError>),
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
                self.screen = Some(Screen::NoVaultFound {
                    path,
                    password: String::new(),
                    confirm: String::new(),
                    error: None,
                    creating: false,
                });
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
            Message::CreatePasswordChanged(new_password) => {
                if let Some(Screen::NoVaultFound { password, .. }) = &mut self.screen {
                    *password = new_password;
                }
                Task::none()
            }
            Message::CreateConfirmChanged(new_confirm) => {
                if let Some(Screen::NoVaultFound { confirm, .. }) = &mut self.screen {
                    *confirm = new_confirm;
                }
                Task::none()
            }
            Message::CreateSubmitted => {
                let password = match &mut self.screen {
                    Some(Screen::NoVaultFound {
                        password,
                        confirm,
                        error,
                        creating,
                        ..
                    }) => {
                        if password != confirm {
                            *error = Some("Passwords don't match".to_string());
                            return Task::none();
                        }
                        *creating = true;
                        password.clone()
                    }
                    _ => return Task::none(),
                };
                Task::perform(
                    async move { Vault::create(&password) },
                    Message::CreateCompleted,
                )
            }
            Message::CreateCompleted(Ok(vault)) => {
                self.screen = Some(Screen::Unlocked {
                    vault,
                    view: UnlockedView::List {
                        query: String::new(),
                    },
                    last_activity: Instant::now(),
                });
                Task::none()
            }
            Message::CreateCompleted(Err(err)) => {
                if let Some(Screen::NoVaultFound {
                    error, creating, ..
                }) = &mut self.screen
                {
                    *error = Some(err.to_string());
                    *creating = false;
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Some(Screen::Locating) => text("Looking for your sekrets file...").into(),
            Some(Screen::NoVaultFound {
                path,
                password,
                confirm,
                error,
                creating,
            }) => {
                let mut col = column![
                    text(format!("No sekrets file found at {}", path.display())),
                    text("Choose a master password to create one:"),
                    text_input("Master password", password)
                        .on_input(Message::CreatePasswordChanged)
                        .secure(true),
                    text_input("Confirm master password", confirm)
                        .on_input(Message::CreateConfirmChanged)
                        .secure(true),
                    button(if *creating {
                        "Creating..."
                    } else {
                        "Create vault"
                    })
                    .on_press(Message::CreateSubmitted),
                ]
                .spacing(10);
                if let Some(err) = error {
                    col = col.push(text(err));
                }
                col.into()
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

    #[test]
    fn create_submitted_with_mismatched_confirm_shows_error_without_calling_vault() {
        let mut app = SekretsApp {
            screen: Some(Screen::NoVaultFound {
                path: PathBuf::from("/tmp/sekrets.enc"),
                password: "hunter2".to_string(),
                confirm: "different".to_string(),
                error: None,
                creating: false,
            }),
        };
        let _ = app.update(Message::CreateSubmitted);
        match &app.screen {
            Some(Screen::NoVaultFound {
                error, creating, ..
            }) => {
                assert!(error.is_some());
                assert!(!creating);
            }
            _ => panic!("expected NoVaultFound screen"),
        }
    }

    #[test]
    fn create_completed_success_transitions_to_unlocked() {
        let mut app = SekretsApp {
            screen: Some(Screen::NoVaultFound {
                path: PathBuf::from("/tmp/sekrets.enc"),
                password: "hunter2".to_string(),
                confirm: "hunter2".to_string(),
                error: None,
                creating: true,
            }),
        };
        sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let vault = sekrets_core::Vault::unlock("hunter2").expect("unlock should succeed");
        let _ = app.update(Message::CreateCompleted(Ok(vault)));
        assert!(matches!(app.screen, Some(Screen::Unlocked { .. })));
    }
}
