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
        clipboard_copied_at: Option<Instant>,
    },
    MigrationPrompt {
        vault: Vault,
    },
}

#[allow(dead_code)]
pub enum UnlockedView {
    List {
        query: String,
    },
    // Fully built out by Task E2 (Detail view: reveal/copy/clipboard); this task
    // only needs the variant to exist so `CredentialSelected` can transition into it.
    Detail {
        key: (String, String),
        revealed: bool,
    },
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
    MigrateAccepted,
    MigrateDeclined,
    MigrateCompleted(Vault, Result<(), VaultError>),
    SearchChanged(String),
    CredentialSelected(String, String),
    RevealToggled,
    CopyPassword(String),
}

/// Returns true once `delay` has elapsed since `copied_at`, as measured against `now`.
/// Pure helper so it can be unit-tested without driving the app's async runtime; the
/// actual clearing (calling `iced::clipboard::write(String::new())`) is wired into
/// Task H1's `Message::Tick` handler, which will call this alongside the auto-lock check.
#[allow(dead_code)]
pub fn should_clear_clipboard(
    copied_at: Instant,
    now: Instant,
    delay: std::time::Duration,
) -> bool {
    now.duration_since(copied_at) >= delay
}

fn enter_unlocked_or_migration(vault: Vault) -> Screen {
    if vault.needs_migration() {
        Screen::MigrationPrompt { vault }
    } else {
        Screen::Unlocked {
            vault,
            view: UnlockedView::List {
                query: String::new(),
            },
            last_activity: Instant::now(),
            clipboard_copied_at: None,
        }
    }
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
                self.screen = Some(enter_unlocked_or_migration(vault));
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
                self.screen = Some(enter_unlocked_or_migration(vault));
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
            Message::MigrateAccepted => {
                let vault = match self.screen.take() {
                    Some(Screen::MigrationPrompt { vault }) => vault,
                    other => {
                        self.screen = other;
                        return Task::none();
                    }
                };
                Task::perform(
                    async move {
                        let mut vault = vault;
                        let result = vault.migrate();
                        (vault, result)
                    },
                    |(vault, result)| Message::MigrateCompleted(vault, result),
                )
            }
            Message::MigrateDeclined => {
                if let Some(Screen::MigrationPrompt { vault }) = self.screen.take() {
                    self.screen = Some(Screen::Unlocked {
                        vault,
                        view: UnlockedView::List {
                            query: String::new(),
                        },
                        last_activity: Instant::now(),
                        clipboard_copied_at: None,
                    });
                }
                Task::none()
            }
            Message::MigrateCompleted(vault, _result) => {
                // Migration failure: the in-memory vault is still usable even if the
                // backup/persist step failed, so proceed to Unlocked rather than strand
                // the user on a dead-end screen. `needs_migration()` correctly stays
                // true on failure (Vault::migrate's `?` short-circuits before clearing
                // the flag), so the user is re-prompted on their next full unlock. A
                // future iteration could surface the failure as a banner in the List
                // view instead of silently proceeding.
                self.screen = Some(Screen::Unlocked {
                    vault,
                    view: UnlockedView::List {
                        query: String::new(),
                    },
                    last_activity: Instant::now(),
                    clipboard_copied_at: None,
                });
                Task::none()
            }
            Message::SearchChanged(new_query) => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::List { query },
                    last_activity,
                    ..
                }) = &mut self.screen
                {
                    *query = new_query;
                    *last_activity = Instant::now();
                }
                Task::none()
            }
            Message::CredentialSelected(account, username) => {
                if let Some(Screen::Unlocked {
                    view,
                    last_activity,
                    ..
                }) = &mut self.screen
                {
                    *view = UnlockedView::Detail {
                        key: (account, username),
                        revealed: false,
                    };
                    *last_activity = Instant::now();
                }
                Task::none()
            }
            Message::RevealToggled => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::Detail { revealed, .. },
                    last_activity,
                    ..
                }) = &mut self.screen
                {
                    *revealed = !*revealed;
                    *last_activity = Instant::now();
                }
                Task::none()
            }
            Message::CopyPassword(password) => {
                if let Some(Screen::Unlocked {
                    last_activity,
                    clipboard_copied_at,
                    ..
                }) = &mut self.screen
                {
                    *last_activity = Instant::now();
                    *clipboard_copied_at = Some(Instant::now());
                }
                iced::clipboard::write(password)
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
            Some(Screen::Unlocked {
                vault,
                view: UnlockedView::List { query },
                ..
            }) => {
                let results = vault.search(query);
                let mut list = column![
                    text_input("Search accounts or usernames", query)
                        .on_input(Message::SearchChanged),
                ]
                .spacing(10);
                for cred in results {
                    list = list.push(
                        button(text(format!("{} — {}", cred.account, cred.username))).on_press(
                            Message::CredentialSelected(cred.account.clone(), cred.username.clone()),
                        ),
                    );
                }
                list.into()
            }
            Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Detail { key, revealed },
                ..
            }) => {
                let creds = vault.search(&key.0);
                let cred = creds.iter().find(|c| c.username == key.1);
                match cred {
                    Some(cred) => {
                        let password_display = if *revealed {
                            cred.password.clone()
                        } else {
                            "••••••••".to_string()
                        };
                        column![
                            text(format!("{} — {}", cred.account, cred.username)),
                            text(password_display),
                            button(if *revealed { "Hide" } else { "Reveal" })
                                .on_press(Message::RevealToggled),
                            button("Copy password")
                                .on_press(Message::CopyPassword(cred.password.clone())),
                        ]
                        .spacing(10)
                        .into()
                    }
                    None => text("Credential not found").into(),
                }
            }
            Some(Screen::MigrationPrompt { .. }) => column![
                text("Your sekrets file uses an older format."),
                text(
                    "It will be upgraded to the new format. A backup of your current file will be saved first."
                ),
                button("Upgrade now").on_press(Message::MigrateAccepted),
                button("Not now").on_press(Message::MigrateDeclined),
            ]
            .spacing(10)
            .into(),
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

    #[test]
    fn unlock_completed_with_needs_migration_shows_migration_prompt() {
        let mut app = SekretsApp {
            screen: Some(Screen::Locked {
                path: PathBuf::from("/tmp/sekrets.enc"),
                password: "foo".to_string(),
                error: None,
                unlocking: true,
            }),
        };
        sekrets_core::encryption::encryptor::encrypt_text(
            "github - username: foo, password: bar",
            "foo",
        )
        .expect("encrypt_text should succeed");
        let vault = sekrets_core::Vault::unlock("foo").expect("unlock should succeed");
        assert!(vault.needs_migration());

        let _ = app.update(Message::UnlockCompleted(Ok(vault)));
        assert!(matches!(app.screen, Some(Screen::MigrationPrompt { .. })));
    }

    #[test]
    fn migrate_declined_goes_to_unlocked_without_migrating() {
        let vault = sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::MigrationPrompt { vault }),
        };

        let _ = app.update(Message::MigrateDeclined);
        match &app.screen {
            Some(Screen::Unlocked { vault, .. }) => assert!(!vault.needs_migration()),
            _ => panic!("expected Unlocked screen"),
        }
    }

    #[test]
    fn search_changed_updates_query_in_list_view() {
        let vault = sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::List {
                    query: String::new(),
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::SearchChanged("git".to_string()));
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::List { query },
                ..
            }) => {
                assert_eq!(query, "git")
            }
            _ => panic!("expected List view"),
        }
    }

    #[test]
    fn migrate_accepted_then_completed_reaches_unlocked_even_on_migrate_error() {
        let vault = sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::MigrationPrompt { vault }),
        };
        // Simulate the async round-trip directly: construct the completion message
        // with an Err result and confirm the screen still recovers to Unlocked.
        let vault_after = match app.screen.take() {
            Some(Screen::MigrationPrompt { vault }) => vault,
            _ => panic!("expected MigrationPrompt"),
        };
        let _ = app.update(Message::MigrateCompleted(
            vault_after,
            Err(sekrets_core::VaultError::Corrupt(
                "simulated failure".to_string(),
            )),
        ));
        assert!(matches!(app.screen, Some(Screen::Unlocked { .. })));
    }

    #[test]
    fn reveal_toggled_flips_revealed_flag() {
        let vault = sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Detail {
                    key: ("github".to_string(), "alice".to_string()),
                    revealed: false,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::RevealToggled);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::Detail { revealed, .. },
                ..
            }) => {
                assert!(revealed)
            }
            _ => panic!("expected Detail view"),
        }
    }

    #[test]
    fn should_clear_clipboard_true_after_delay_elapsed() {
        let copied_at = Instant::now() - std::time::Duration::from_secs(31);
        assert!(should_clear_clipboard(
            copied_at,
            Instant::now(),
            std::time::Duration::from_secs(30)
        ));
    }

    #[test]
    fn should_clear_clipboard_false_before_delay_elapsed() {
        let copied_at = Instant::now();
        assert!(!should_clear_clipboard(
            copied_at,
            Instant::now(),
            std::time::Duration::from_secs(30)
        ));
    }
}
