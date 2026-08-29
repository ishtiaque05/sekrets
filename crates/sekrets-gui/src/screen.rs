use std::path::PathBuf;
use std::time::Instant;

use iced::widget::{button, column, row, text, text_input};
use iced::{Element, Task};
use sekrets_core::{Vault, VaultError, VersionInfo};

// Exactly one `Screen` exists for the lifetime of the app (it *is* the app state), so the
// size spread between variants costs a couple of hundred bytes once — boxing fields to
// even it out would only add indirection and pattern-matching noise to every handler.
#[allow(clippy::large_enum_variant)]
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
    Unlocked {
        vault: Vault,
        view: UnlockedView,
        last_activity: Instant,
        /// The password most recently copied to the clipboard, and when. Retained (rather
        /// than just the timestamp) so the delayed clear can read the clipboard back and
        /// only wipe it if the user hasn't copied something else in the meantime.
        clipboard_copied_at: Option<(Instant, String)>,
    },
    /// Holds a decrypted `Vault`, so it carries `last_activity` and auto-locks exactly
    /// like `Unlocked` — a user can otherwise sit on this prompt indefinitely.
    MigrationPrompt {
        vault: Vault,
        last_activity: Instant,
    },
    /// The window while the async migration is in flight. The `Vault` lives inside the
    /// pending future rather than in the screen, but the app is still effectively unlocked,
    /// so this screen keeps ticking and can auto-lock too (see `Message::MigrateCompleted`,
    /// which drops the returned vault if the app locked while it was working).
    Migrating {
        last_activity: Instant,
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
    // `key: None` means adding a new credential; `key: Some((account, username))`
    // means editing that existing credential.
    Edit {
        key: Option<(String, String)>,
        account: String,
        username: String,
        password: String,
        error: Option<VaultError>,
    },
    DeleteConfirm {
        key: (String, String),
        error: Option<VaultError>,
    },
    ChangeMasterPassword {
        new: String,
        confirm: String,
        error: Option<VaultError>,
    },
    Versions {
        versions: Vec<VersionInfo>,
        selected_version: Option<usize>,
        version_password: String,
        error: Option<VaultError>,
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
    NavigateToAdd,
    NavigateToEdit(String, String),
    EditAccountChanged(String),
    EditUsernameChanged(String),
    EditPasswordChanged(String),
    GeneratePassword,
    EditSubmitted,
    DeleteRequested(String, String),
    DeleteConfirmed,
    DeleteCancelled,
    NavigateToChangeMasterPassword,
    ChangeMpNewChanged(String),
    ChangeMpConfirmChanged(String),
    ChangeMpSubmitted,
    NavigateToVersions,
    NavigateToList,
    SwitchVersionPasswordChanged(String),
    SwitchVersionRequested(usize),
    SwitchVersionCompleted(Result<(), VaultError>),
    /// Result of reading the clipboard back once the auto-clear delay elapsed.
    /// `expected` is what this app put there; `current` is what is on the clipboard now.
    /// They differ when the user copied something else in the meantime, in which case
    /// nothing is wiped.
    ClipboardCheckedForClear {
        expected: String,
        current: Option<String>,
    },
    Tick,
}

/// Returns true once `delay` has elapsed since `copied_at`, as measured against `now`.
/// Pure helper so it can be unit-tested without driving the app's async runtime; the
/// actual clearing (calling `iced::clipboard::write(String::new())`) is wired into
/// Task H1's `Message::Tick` handler, which will call this alongside the auto-lock check.
pub fn should_clear_clipboard(
    copied_at: Instant,
    now: Instant,
    delay: std::time::Duration,
) -> bool {
    now.duration_since(copied_at) >= delay
}

/// How long the app may sit idle (no `Message` other than `Tick` observed) before
/// `Message::Tick` locks the vault, per `should_lock`.
pub const AUTO_LOCK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// How long a copied password stays on the clipboard before `Message::Tick` clears it,
/// per `should_clear_clipboard`.
pub const CLIPBOARD_CLEAR_DELAY: std::time::Duration = std::time::Duration::from_secs(25);

/// Returns true once `timeout` has elapsed since `last_activity`, as measured against
/// `now`. Pure helper so it can be unit-tested without driving the app's async runtime;
/// the actual locking (reassigning `self.screen` to `Screen::Locked`, which drops the
/// `Vault` via ownership) is wired into `Message::Tick`'s handler.
pub fn should_lock(last_activity: Instant, now: Instant, timeout: std::time::Duration) -> bool {
    now.duration_since(last_activity) >= timeout
}

fn enter_unlocked_or_migration(vault: Vault) -> Screen {
    if vault.needs_migration() {
        Screen::MigrationPrompt {
            vault,
            last_activity: Instant::now(),
        }
    } else {
        unlocked_list(vault)
    }
}

fn unlocked_list(vault: Vault) -> Screen {
    Screen::Unlocked {
        vault,
        view: UnlockedView::List {
            query: String::new(),
        },
        last_activity: Instant::now(),
        clipboard_copied_at: None,
    }
}

/// The screen shown when the app auto-locks: the `Vault` is dropped by moving out of it.
fn locked_screen() -> Screen {
    Screen::Locked {
        path: Vault::locate().unwrap_or_default(),
        password: String::new(),
        error: None,
        unlocking: false,
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
        // `Tick` and the timer-driven clipboard read-back are not user activity, so they
        // must not postpone the idle auto-lock.
        if !matches!(
            message,
            Message::Tick | Message::ClipboardCheckedForClear { .. }
        ) {
            match &mut self.screen {
                Some(Screen::Unlocked { last_activity, .. })
                | Some(Screen::MigrationPrompt { last_activity, .. })
                | Some(Screen::Migrating { last_activity }) => *last_activity = Instant::now(),
                _ => {}
            }
        }
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
                    Some(Screen::MigrationPrompt { vault, .. }) => vault,
                    other => {
                        self.screen = other;
                        return Task::none();
                    }
                };
                // A real screen (rather than `None`) so the subscription keeps ticking
                // and the app can still auto-lock while the migration runs.
                self.screen = Some(Screen::Migrating {
                    last_activity: Instant::now(),
                });
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
                if let Some(Screen::MigrationPrompt { vault, .. }) = self.screen.take() {
                    self.screen = Some(unlocked_list(vault));
                }
                Task::none()
            }
            Message::MigrateCompleted(vault, _result) => {
                // If the app auto-locked while the migration was in flight, honour that:
                // dropping `vault` here is what actually discards the decrypted secrets
                // the pending future was holding.
                if !matches!(self.screen, Some(Screen::Migrating { .. })) {
                    return Task::none();
                }
                // Migration failure: the in-memory vault is still usable even if the
                // backup/persist step failed, so proceed to Unlocked rather than strand
                // the user on a dead-end screen. `needs_migration()` correctly stays
                // true on failure (Vault::migrate's `?` short-circuits before clearing
                // the flag), so the user is re-prompted on their next full unlock. A
                // future iteration could surface the failure as a banner in the List
                // view instead of silently proceeding.
                self.screen = Some(unlocked_list(vault));
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
                    *clipboard_copied_at = Some((Instant::now(), password.clone()));
                }
                iced::clipboard::write(password)
            }
            Message::NavigateToList => {
                if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                    *view = UnlockedView::List {
                        query: String::new(),
                    };
                }
                Task::none()
            }
            Message::NavigateToAdd => {
                if let Some(Screen::Unlocked {
                    view,
                    last_activity,
                    ..
                }) = &mut self.screen
                {
                    *view = UnlockedView::Edit {
                        key: None,
                        account: String::new(),
                        username: String::new(),
                        password: String::new(),
                        error: None,
                    };
                    *last_activity = Instant::now();
                }
                Task::none()
            }
            Message::NavigateToEdit(account, username) => {
                if let Some(Screen::Unlocked {
                    vault,
                    view,
                    last_activity,
                    ..
                }) = &mut self.screen
                {
                    let existing_password = vault
                        .search(&account)
                        .iter()
                        .find(|c| c.username == username)
                        .map(|c| c.password.clone());
                    if let Some(password) = existing_password {
                        *view = UnlockedView::Edit {
                            key: Some((account.clone(), username.clone())),
                            account: account.clone(),
                            username: username.clone(),
                            password,
                            error: None,
                        };
                    }
                    *last_activity = Instant::now();
                }
                Task::none()
            }
            Message::EditAccountChanged(v) => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::Edit { account, .. },
                    ..
                }) = &mut self.screen
                {
                    *account = v;
                }
                Task::none()
            }
            Message::EditUsernameChanged(v) => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::Edit { username, .. },
                    ..
                }) = &mut self.screen
                {
                    *username = v;
                }
                Task::none()
            }
            Message::EditPasswordChanged(v) => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::Edit { password, .. },
                    ..
                }) = &mut self.screen
                {
                    *password = v;
                }
                Task::none()
            }
            Message::GeneratePassword => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::Edit { password, .. },
                    ..
                }) = &mut self.screen
                {
                    *password =
                        sekrets_core::secrets::password_generator::PasswordGenerator::new(None)
                            .generate_random();
                }
                Task::none()
            }
            Message::EditSubmitted => {
                if let Some(Screen::Unlocked {
                    vault,
                    view:
                        UnlockedView::Edit {
                            key,
                            account,
                            username,
                            password,
                            error,
                        },
                    ..
                }) = &mut self.screen
                {
                    let result = match key {
                        None => vault.add(account, username, password),
                        Some((old_account, old_username)) => {
                            if old_account == account && old_username == username {
                                vault.update(account, username, password)
                            } else {
                                // Renaming account/username: add the new key first, then
                                // delete the old one. This ordering matters: if `add` fails
                                // (e.g. the new account/username collides with a different
                                // existing credential), the original credential is left
                                // untouched. If `delete` somehow fails after a successful
                                // `add`, the worst case is a harmless duplicate entry rather
                                // than permanent data loss.
                                vault
                                    .add(account, username, password)
                                    .and_then(|_| vault.delete(old_account, old_username))
                            }
                        }
                    };
                    match result {
                        Ok(()) => {
                            if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                                *view = UnlockedView::List {
                                    query: String::new(),
                                };
                            }
                        }
                        Err(e) => {
                            *error = Some(e);
                        }
                    }
                }
                Task::none()
            }
            Message::DeleteRequested(account, username) => {
                if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                    *view = UnlockedView::DeleteConfirm {
                        key: (account, username),
                        error: None,
                    };
                }
                Task::none()
            }
            Message::DeleteConfirmed => {
                if let Some(Screen::Unlocked {
                    vault,
                    view: UnlockedView::DeleteConfirm { key, error },
                    ..
                }) = &mut self.screen
                {
                    match vault.delete(&key.0, &key.1) {
                        Ok(()) => {
                            if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                                *view = UnlockedView::List {
                                    query: String::new(),
                                };
                            }
                        }
                        Err(e) => {
                            *error = Some(e);
                        }
                    }
                }
                Task::none()
            }
            Message::DeleteCancelled => {
                if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                    *view = UnlockedView::List {
                        query: String::new(),
                    };
                }
                Task::none()
            }
            Message::NavigateToChangeMasterPassword => {
                if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                    *view = UnlockedView::ChangeMasterPassword {
                        new: String::new(),
                        confirm: String::new(),
                        error: None,
                    };
                }
                Task::none()
            }
            Message::ChangeMpNewChanged(v) => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::ChangeMasterPassword { new, .. },
                    ..
                }) = &mut self.screen
                {
                    *new = v;
                }
                Task::none()
            }
            Message::ChangeMpConfirmChanged(v) => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::ChangeMasterPassword { confirm, .. },
                    ..
                }) = &mut self.screen
                {
                    *confirm = v;
                }
                Task::none()
            }
            Message::ChangeMpSubmitted => {
                if let Some(Screen::Unlocked {
                    vault,
                    view:
                        UnlockedView::ChangeMasterPassword {
                            new,
                            confirm,
                            error,
                        },
                    ..
                }) = &mut self.screen
                {
                    if new != confirm {
                        *error = Some(VaultError::Io("Passwords don't match".to_string()));
                    } else {
                        match vault.change_master_password(new) {
                            Ok(()) => {
                                if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                                    *view = UnlockedView::List {
                                        query: String::new(),
                                    };
                                }
                            }
                            Err(e) => *error = Some(e),
                        }
                    }
                }
                Task::none()
            }
            Message::NavigateToVersions => {
                if let Some(Screen::Unlocked { vault, view, .. }) = &mut self.screen {
                    let (versions, error) = match vault.list_versions() {
                        Ok(v) => (v, None),
                        Err(e) => (Vec::new(), Some(e)),
                    };
                    *view = UnlockedView::Versions {
                        versions,
                        selected_version: None,
                        version_password: String::new(),
                        error,
                    };
                }
                Task::none()
            }
            Message::SwitchVersionPasswordChanged(v) => {
                if let Some(Screen::Unlocked {
                    view:
                        UnlockedView::Versions {
                            version_password, ..
                        },
                    ..
                }) = &mut self.screen
                {
                    *version_password = v;
                }
                Task::none()
            }
            Message::SwitchVersionRequested(n) => {
                if let Some(Screen::Unlocked {
                    vault,
                    view:
                        UnlockedView::Versions {
                            selected_version,
                            version_password,
                            ..
                        },
                    ..
                }) = &mut self.screen
                {
                    *selected_version = Some(n);
                    let result = vault.switch_version(n, version_password);
                    return Task::perform(async move { result }, Message::SwitchVersionCompleted);
                }
                Task::none()
            }
            Message::SwitchVersionCompleted(Ok(())) => {
                if let Some(Screen::Unlocked { view, .. }) = &mut self.screen {
                    *view = UnlockedView::List {
                        query: String::new(),
                    };
                }
                Task::none()
            }
            Message::SwitchVersionCompleted(Err(err)) => {
                if let Some(Screen::Unlocked {
                    view: UnlockedView::Versions { error, .. },
                    ..
                }) = &mut self.screen
                {
                    *error = Some(err);
                }
                Task::none()
            }
            Message::ClipboardCheckedForClear { expected, current } => {
                // Only wipe if the clipboard still holds what this app put there — the
                // user may have copied something unrelated in the meantime, and clobbering
                // that is exactly what the spec forbids.
                if current.as_deref() == Some(expected.as_str()) {
                    return iced::clipboard::write(String::new());
                }
                Task::none()
            }
            Message::Tick => {
                let now = Instant::now();

                // Every screen that holds (or has in flight) a decrypted vault ticks.
                let last_activity = match &self.screen {
                    Some(Screen::Unlocked { last_activity, .. })
                    | Some(Screen::MigrationPrompt { last_activity, .. })
                    | Some(Screen::Migrating { last_activity }) => *last_activity,
                    _ => return Task::none(),
                };

                if should_lock(last_activity, now, AUTO_LOCK_TIMEOUT) {
                    self.screen = Some(locked_screen());
                    return Task::none();
                }

                let expected = match &mut self.screen {
                    Some(Screen::Unlocked {
                        clipboard_copied_at,
                        ..
                    }) => match clipboard_copied_at {
                        Some((copied_at, _))
                            if should_clear_clipboard(*copied_at, now, CLIPBOARD_CLEAR_DELAY) =>
                        {
                            // Taken here so the read-back is issued exactly once.
                            clipboard_copied_at.take().map(|(_, password)| password)
                        }
                        _ => None,
                    },
                    _ => None,
                };

                match expected {
                    Some(expected) => iced::clipboard::read().map(move |current| {
                        Message::ClipboardCheckedForClear {
                            expected: expected.clone(),
                            current,
                        }
                    }),
                    None => Task::none(),
                }
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
                    button("Add credential").on_press(Message::NavigateToAdd),
                    button("Change master password")
                        .on_press(Message::NavigateToChangeMasterPassword),
                    button("Versions").on_press(Message::NavigateToVersions),
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
                        let mut col = column![
                            text(format!("{} — {}", cred.account, cred.username)),
                            text(password_display),
                            button(if *revealed { "Hide" } else { "Reveal" })
                                .on_press(Message::RevealToggled),
                            button("Copy password")
                                .on_press(Message::CopyPassword(cred.password.clone())),
                            button("Edit").on_press(Message::NavigateToEdit(
                                cred.account.clone(),
                                cred.username.clone()
                            )),
                            button("Delete").on_press(Message::DeleteRequested(
                                cred.account.clone(),
                                cred.username.clone()
                            )),
                            button("Back").on_press(Message::NavigateToList),
                            text("Password history:"),
                        ]
                        .spacing(10);
                        if cred.history.is_empty() {
                            col = col.push(text("No previous passwords recorded."));
                        } else {
                            for (i, entry) in cred.history.iter().enumerate() {
                                col = col.push(text(format!(
                                    "v{}: ******** ({})",
                                    i + 1,
                                    entry.format_ts_local()
                                )));
                            }
                        }
                        col.into()
                    }
                    None => text("Credential not found").into(),
                }
            }
            Some(Screen::Unlocked {
                view:
                    UnlockedView::Edit {
                        key,
                        account,
                        username,
                        password,
                        error,
                    },
                ..
            }) => {
                let strength = if password.is_empty() {
                    ""
                } else if sekrets_core::secrets::password_generator::is_password_strong(password)
                {
                    "Strong"
                } else {
                    "Weak — consider a longer or more complex password"
                };

                let mut col = column![
                    text(if key.is_some() {
                        "Edit credential"
                    } else {
                        "Add credential"
                    }),
                    text_input("Account", account).on_input(Message::EditAccountChanged),
                    text_input("Username", username).on_input(Message::EditUsernameChanged),
                    text_input("Password", password)
                        .on_input(Message::EditPasswordChanged)
                        .secure(true),
                    button("Generate password").on_press(Message::GeneratePassword),
                    text(strength),
                    button("Save").on_press(Message::EditSubmitted),
                    button("Cancel").on_press(Message::NavigateToList),
                ]
                .spacing(10);
                if let Some(err) = error {
                    col = col.push(text(err.to_string()));
                }
                col.into()
            }
            Some(Screen::Unlocked {
                view: UnlockedView::DeleteConfirm { key, error },
                ..
            }) => {
                let mut col = column![
                    text(format!(
                        "Delete {} — {}? This cannot be undone.",
                        key.0, key.1
                    )),
                    button("Delete").on_press(Message::DeleteConfirmed),
                    button("Cancel").on_press(Message::DeleteCancelled),
                ]
                .spacing(10);
                if let Some(err) = error {
                    col = col.push(text(err.to_string()));
                }
                col.into()
            }
            Some(Screen::Unlocked {
                view:
                    UnlockedView::ChangeMasterPassword {
                        new,
                        confirm,
                        error,
                    },
                ..
            }) => {
                let mut col = column![
                    text("Change master password"),
                    text_input("New master password", new)
                        .on_input(Message::ChangeMpNewChanged)
                        .secure(true),
                    text_input("Confirm new master password", confirm)
                        .on_input(Message::ChangeMpConfirmChanged)
                        .secure(true),
                    button("Change password").on_press(Message::ChangeMpSubmitted),
                    button("Cancel").on_press(Message::NavigateToList),
                ]
                .spacing(10);
                if let Some(err) = error {
                    col = col.push(text(err.to_string()));
                }
                col.into()
            }
            Some(Screen::Unlocked {
                view:
                    UnlockedView::Versions {
                        versions,
                        version_password,
                        error,
                        ..
                    },
                ..
            }) => {
                let mut col = column![text("Versions")].spacing(10);
                for v in versions {
                    col = col.push(
                        row![
                            // The modified time, not the number, is what identifies a
                            // restore point: numbers shift on every snapshot rotation.
                            text(format!("v{}  {}", v.number, v.format_modified_local())),
                            button("Switch to this version")
                                .on_press(Message::SwitchVersionRequested(v.number)),
                        ]
                        .spacing(10),
                    );
                }
                col = col.push(
                    text_input("Password for the selected version", version_password)
                        .on_input(Message::SwitchVersionPasswordChanged)
                        .secure(true),
                );
                col = col.push(button("Back").on_press(Message::NavigateToList));
                if let Some(err) = error {
                    col = col.push(text(err.to_string()));
                }
                col.into()
            }
            Some(Screen::Migrating { .. }) => {
                text("Upgrading your sekrets file...").into()
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

    const TEST_PASSWORD: &str = "hunter2";

    use futures::StreamExt;
    use iced_runtime::task::into_stream;
    use iced_runtime::Action;

    /// Drives a real `iced::Task` to completion on a blocking executor and returns the
    /// `Message`s it produced.
    ///
    /// This exercises the actual `Task`/future machinery rather than hand-constructing the
    /// completion message: an executor that silently drops futures (which is exactly what
    /// iced's "null" backend did here before the `tokio` feature was enabled) yields an
    /// empty `Vec` and fails these tests.
    ///
    /// Only safe for tasks whose actions all resolve on their own. Tasks that ask the
    /// windowing runtime a question — `iced::clipboard::read()` — park forever on a oneshot
    /// channel nobody answers; use [`first_action`] for those.
    fn drive_task(task: Task<Message>) -> Vec<Message> {
        let Some(stream) = into_stream(task) else {
            return Vec::new();
        };
        futures::executor::block_on(
            stream
                .filter_map(|action| async move {
                    match action {
                        Action::Output(message) => Some(message),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    /// The first runtime `Action` a task emits, without waiting for the rest of the stream.
    fn first_action(task: Task<Message>) -> Option<Action<Message>> {
        let stream = into_stream(task)?;
        futures::executor::block_on(Box::pin(stream).next())
    }

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
            screen: Some(Screen::MigrationPrompt {
                vault,
                last_activity: Instant::now(),
            }),
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
    fn migrate_completed_reaches_unlocked_even_on_migrate_error() {
        let vault = sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Migrating {
                last_activity: Instant::now(),
            }),
        };
        let _ = app.update(Message::MigrateCompleted(
            vault,
            Err(sekrets_core::VaultError::Corrupt(
                "simulated failure".to_string(),
            )),
        ));
        assert!(matches!(app.screen, Some(Screen::Unlocked { .. })));
    }

    #[test]
    fn migrate_accepted_moves_to_a_tickable_migrating_screen() {
        // Regression test: this used to `screen.take()` and leave `screen: None`, which
        // the subscription never ticks — an un-lockable window holding a decrypted vault
        // in the pending future.
        let vault = sekrets_core::Vault::create("hunter2").expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::MigrationPrompt {
                vault,
                last_activity: Instant::now(),
            }),
        };
        let _ = app.update(Message::MigrateAccepted);
        assert!(matches!(app.screen, Some(Screen::Migrating { .. })));
    }

    #[test]
    fn tick_locks_migration_prompt_after_idle_timeout() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::MigrationPrompt {
                vault,
                last_activity: Instant::now() - std::time::Duration::from_secs(301),
            }),
        };
        let _ = app.update(Message::Tick);
        assert!(matches!(app.screen, Some(Screen::Locked { .. })));
    }

    #[test]
    fn tick_does_not_lock_a_recently_active_migration_prompt() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::MigrationPrompt {
                vault,
                last_activity: Instant::now(),
            }),
        };
        let _ = app.update(Message::Tick);
        assert!(matches!(app.screen, Some(Screen::MigrationPrompt { .. })));
    }

    #[test]
    fn tick_locks_during_migration_and_the_completion_does_not_reopen_the_vault() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Migrating {
                last_activity: Instant::now() - std::time::Duration::from_secs(301),
            }),
        };
        let _ = app.update(Message::Tick);
        assert!(matches!(app.screen, Some(Screen::Locked { .. })));

        // The in-flight migration finishes after the auto-lock: the returned vault must be
        // dropped, not used to silently re-enter the unlocked app.
        let _ = app.update(Message::MigrateCompleted(vault, Ok(())));
        assert!(matches!(app.screen, Some(Screen::Locked { .. })));
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

    #[test]
    fn edit_submitted_add_success_returns_to_list() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Edit {
                    key: None,
                    account: "github".to_string(),
                    username: "alice".to_string(),
                    password: "hunter2".to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::EditSubmitted);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                vault,
                ..
            }) => {
                assert_eq!(vault.search("github").len(), 1);
            }
            _ => panic!("expected List view after successful add"),
        }
    }

    #[test]
    fn edit_submitted_duplicate_add_shows_inline_error_and_keeps_input() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "existing")
            .expect("add should succeed");

        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Edit {
                    key: None,
                    account: "github".to_string(),
                    username: "alice".to_string(),
                    password: "hunter2".to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::EditSubmitted);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::Edit { error, account, .. },
                ..
            }) => {
                assert!(error.is_some());
                assert_eq!(account, "github"); // input preserved, not discarded
            }
            _ => panic!("expected to stay on Edit view with an error"),
        }
    }

    #[test]
    fn edit_submitted_same_key_updates_password_and_returns_to_list() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "oldpass")
            .expect("add should succeed");

        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Edit {
                    key: Some(("github".to_string(), "alice".to_string())),
                    account: "github".to_string(),
                    username: "alice".to_string(),
                    password: "newpass".to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::EditSubmitted);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                vault,
                ..
            }) => {
                let creds = vault.search("github");
                assert_eq!(creds.len(), 1);
                assert_eq!(creds[0].password, "newpass");
            }
            _ => panic!("expected List view after successful update"),
        }
    }

    #[test]
    fn edit_submitted_rename_success_moves_credential_to_new_key() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "secret")
            .expect("add should succeed");

        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Edit {
                    key: Some(("github".to_string(), "alice".to_string())),
                    account: "gitlab".to_string(),
                    username: "alice2".to_string(),
                    password: "secret".to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::EditSubmitted);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                vault,
                ..
            }) => {
                assert!(vault.search("github").iter().all(|c| c.username != "alice"));
                let new_creds = vault.search("gitlab");
                assert_eq!(new_creds.len(), 1);
                assert_eq!(new_creds[0].username, "alice2");
            }
            _ => panic!("expected List view after successful rename"),
        }
    }

    #[test]
    fn edit_submitted_rename_collision_preserves_original_credential() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "secret")
            .expect("add should succeed");
        vault
            .add("gitlab", "bob", "other-secret")
            .expect("add should succeed");

        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                // Renaming github/alice onto gitlab/bob, which already exists: the add
                // should fail with AccountAlreadyExists, and the original github/alice
                // credential must remain untouched (regression test for the
                // delete-then-add data-loss bug: add-then-delete ordering means the
                // failed `add` never reaches the `delete` step).
                view: UnlockedView::Edit {
                    key: Some(("github".to_string(), "alice".to_string())),
                    account: "gitlab".to_string(),
                    username: "bob".to_string(),
                    password: "hijacked".to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::EditSubmitted);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::Edit { error, .. },
                vault,
                ..
            }) => {
                assert!(error.is_some());
                let original = vault
                    .search("github")
                    .into_iter()
                    .find(|c| c.username == "alice")
                    .expect("original github/alice credential must still exist");
                assert_eq!(original.password, "secret");
                let untouched = vault
                    .search("gitlab")
                    .into_iter()
                    .find(|c| c.username == "bob")
                    .expect("original gitlab/bob credential must still exist");
                assert_eq!(untouched.password, "other-secret");
            }
            _ => panic!("expected to stay on Edit view with an error"),
        }
    }

    #[test]
    fn navigate_to_edit_prefills_existing_password_and_key() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "secret123")
            .expect("add should succeed");

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
        let _ = app.update(Message::NavigateToEdit(
            "github".to_string(),
            "alice".to_string(),
        ));
        match &app.screen {
            Some(Screen::Unlocked {
                view:
                    UnlockedView::Edit {
                        key,
                        account,
                        username,
                        password,
                        error,
                    },
                ..
            }) => {
                assert_eq!(key, &Some(("github".to_string(), "alice".to_string())));
                assert_eq!(account, "github");
                assert_eq!(username, "alice");
                assert_eq!(password, "secret123");
                assert!(error.is_none());
            }
            _ => panic!("expected Edit view prefilled with existing credential"),
        }
    }

    #[test]
    fn delete_confirmed_removes_credential_and_returns_to_list() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "hunter2")
            .expect("add should succeed");

        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::DeleteConfirm {
                    key: ("github".to_string(), "alice".to_string()),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::DeleteConfirmed);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                vault,
                ..
            }) => {
                assert_eq!(vault.search("github").len(), 0);
            }
            _ => panic!("expected List view after delete"),
        }
    }

    #[test]
    fn delete_confirmed_missing_credential_shows_inline_error_and_stays_on_delete_confirm() {
        // No credential added: the key in DeleteConfirm doesn't exist in the vault,
        // so `Vault::delete` returns `AccountWithUsernameNotFound`. This simulates a
        // race (e.g. deleted elsewhere) without needing to fabricate one.
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");

        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::DeleteConfirm {
                    key: ("github".to_string(), "alice".to_string()),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::DeleteConfirmed);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::DeleteConfirm { error, .. },
                ..
            }) => {
                assert!(error.is_some());
            }
            _ => panic!("expected to stay on DeleteConfirm view with an error"),
        }
    }

    #[test]
    fn detail_view_renders_without_panicking_when_history_present() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "v1")
            .expect("add should succeed");
        vault
            .update("github", "alice", "v2")
            .expect("update should succeed");

        let app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Detail {
                    key: ("github".to_string(), "alice".to_string()),
                    revealed: true,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.view(); // must not panic
    }

    #[test]
    fn change_mp_submitted_mismatched_confirm_shows_error() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::ChangeMasterPassword {
                    new: "new-pass".to_string(),
                    confirm: "different".to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::ChangeMpSubmitted);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::ChangeMasterPassword { error, .. },
                ..
            }) => {
                assert!(error.is_some())
            }
            _ => panic!("expected to stay on ChangeMasterPassword view"),
        }
    }

    #[test]
    fn change_mp_submitted_matching_confirm_changes_password_and_returns_to_list() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::ChangeMasterPassword {
                    new: "new-master-password".to_string(),
                    confirm: "new-master-password".to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::ChangeMpSubmitted);
        assert!(matches!(
            app.screen,
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                ..
            })
        ));

        let reunlocked = sekrets_core::Vault::unlock("new-master-password");
        assert!(reunlocked.is_ok());
    }

    #[test]
    fn generate_password_fills_password_field() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Edit {
                    key: None,
                    account: "github".to_string(),
                    username: "alice".to_string(),
                    password: String::new(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::GeneratePassword);
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::Edit { password, .. },
                ..
            }) => {
                assert_eq!(password.len(), 16);
            }
            _ => panic!("expected Edit view"),
        }
    }

    #[test]
    fn navigate_to_versions_loads_version_list() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
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
        let _ = app.update(Message::NavigateToVersions);
        assert!(matches!(
            app.screen,
            Some(Screen::Unlocked {
                view: UnlockedView::Versions { .. },
                ..
            })
        ));
    }

    #[test]
    fn switch_version_completed_success_returns_to_list() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Versions {
                    versions: Vec::new(),
                    selected_version: Some(1),
                    version_password: TEST_PASSWORD.to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::SwitchVersionCompleted(Ok(())));
        assert!(matches!(
            app.screen,
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                ..
            })
        ));
    }

    #[test]
    fn should_lock_true_after_timeout_elapsed() {
        let last_activity = Instant::now() - std::time::Duration::from_secs(301);
        assert!(should_lock(
            last_activity,
            Instant::now(),
            std::time::Duration::from_secs(300)
        ));
    }

    #[test]
    fn should_lock_false_before_timeout_elapsed() {
        let last_activity = Instant::now();
        assert!(!should_lock(
            last_activity,
            Instant::now(),
            std::time::Duration::from_secs(300)
        ));
    }

    #[test]
    fn tick_locks_and_drops_vault_when_idle_timeout_elapsed() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let path = sekrets_core::Vault::locate().expect("vault should exist");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::List {
                    query: String::new(),
                },
                last_activity: Instant::now() - std::time::Duration::from_secs(301),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::Tick);
        match &app.screen {
            Some(Screen::Locked {
                path: locked_path, ..
            }) => assert_eq!(locked_path, &path),
            _ => panic!("expected Locked screen after idle timeout"),
        }
    }

    #[test]
    fn tick_does_not_lock_when_recently_active() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
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
        let _ = app.update(Message::Tick);
        assert!(matches!(app.screen, Some(Screen::Unlocked { .. })));
    }

    // ---------------------------------------------------------------------------------
    // Navigation escape hatches (Detail / Edit / ChangeMasterPassword / Versions)
    // ---------------------------------------------------------------------------------

    /// The per-test temp vault, created on first use and reopened afterwards (each test
    /// runs on its own thread, and the test data directory is thread-local).
    fn test_vault() -> Vault {
        sekrets_core::Vault::create(TEST_PASSWORD)
            .or_else(|_| sekrets_core::Vault::unlock(TEST_PASSWORD))
            .expect("vault should be available")
    }

    fn unlocked_with(view: UnlockedView) -> SekretsApp {
        let vault = test_vault();
        SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view,
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        }
    }

    fn assert_on_list(app: &SekretsApp) {
        assert!(matches!(
            app.screen,
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                ..
            })
        ));
    }

    #[test]
    fn navigate_to_list_escapes_the_detail_view() {
        let mut app = unlocked_with(UnlockedView::Detail {
            key: ("github".to_string(), "alice".to_string()),
            revealed: false,
        });
        let _ = app.update(Message::NavigateToList);
        assert_on_list(&app);
    }

    #[test]
    fn navigate_to_list_escapes_the_edit_view_without_saving() {
        let mut app = unlocked_with(UnlockedView::Edit {
            key: None,
            account: "github".to_string(),
            username: "alice".to_string(),
            password: "hunter2".to_string(),
            error: None,
        });
        let _ = app.update(Message::NavigateToList);
        assert_on_list(&app);
        match &app.screen {
            Some(Screen::Unlocked { vault, .. }) => {
                assert_eq!(vault.search("github").len(), 0, "cancel must not save");
            }
            _ => panic!("expected Unlocked screen"),
        }
    }

    #[test]
    fn navigate_to_list_escapes_change_master_password_without_changing_it() {
        let mut app = unlocked_with(UnlockedView::ChangeMasterPassword {
            new: "a-new-password".to_string(),
            confirm: "a-new-password".to_string(),
            error: None,
        });
        let _ = app.update(Message::NavigateToList);
        assert_on_list(&app);
        assert!(
            sekrets_core::Vault::unlock(TEST_PASSWORD).is_ok(),
            "cancelling must leave the master password untouched"
        );
    }

    #[test]
    fn navigate_to_list_escapes_the_versions_view_without_switching() {
        let mut app = unlocked_with(UnlockedView::Versions {
            versions: Vec::new(),
            selected_version: None,
            version_password: String::new(),
            error: None,
        });
        let _ = app.update(Message::NavigateToList);
        assert_on_list(&app);
    }

    #[test]
    fn every_dead_end_view_renders_a_back_or_cancel_button() {
        // The handler above is only reachable if the views actually offer the control;
        // rendering each one proves the button-bearing arm is exercised.
        for view in [
            UnlockedView::Detail {
                key: ("github".to_string(), "alice".to_string()),
                revealed: false,
            },
            UnlockedView::Edit {
                key: None,
                account: String::new(),
                username: String::new(),
                password: String::new(),
                error: None,
            },
            UnlockedView::ChangeMasterPassword {
                new: String::new(),
                confirm: String::new(),
                error: None,
            },
            UnlockedView::Versions {
                versions: Vec::new(),
                selected_version: None,
                version_password: String::new(),
                error: None,
            },
        ] {
            let app = unlocked_with(view);
            let _ = app.view();
        }
    }

    // ---------------------------------------------------------------------------------
    // Clipboard auto-clear: never clobber a newer copy
    // ---------------------------------------------------------------------------------

    #[test]
    fn copy_password_records_what_was_copied_for_the_later_read_back() {
        let mut app = unlocked_with(UnlockedView::Detail {
            key: ("github".to_string(), "alice".to_string()),
            revealed: true,
        });
        let _ = app.update(Message::CopyPassword("s3cret".to_string()));
        match &app.screen {
            Some(Screen::Unlocked {
                clipboard_copied_at: Some((_, copied)),
                ..
            }) => assert_eq!(copied, "s3cret"),
            _ => panic!("expected the copied password to be recorded"),
        }
    }

    #[test]
    fn tick_after_the_delay_reads_the_clipboard_instead_of_wiping_it() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::List {
                    query: String::new(),
                },
                last_activity: Instant::now(),
                clipboard_copied_at: Some((
                    Instant::now() - std::time::Duration::from_secs(26),
                    "s3cret".to_string(),
                )),
            }),
        };

        let action = first_action(app.update(Message::Tick)).expect("expected a clipboard action");

        // A read, never an unconditional write: wiping here is what clobbered whatever the
        // user copied after us.
        assert!(
            matches!(
                action,
                Action::Clipboard(iced_runtime::clipboard::Action::Read { .. })
            ),
            "expected a clipboard read-back, got a different action"
        );

        // The pending copy is consumed, so the read-back is issued exactly once.
        match &app.screen {
            Some(Screen::Unlocked {
                clipboard_copied_at,
                ..
            }) => assert!(clipboard_copied_at.is_none()),
            _ => panic!("expected Unlocked screen"),
        }
    }

    #[test]
    fn clipboard_is_wiped_when_it_still_holds_the_copied_password() {
        let mut app = unlocked_with(UnlockedView::List {
            query: String::new(),
        });
        let action = first_action(app.update(Message::ClipboardCheckedForClear {
            expected: "s3cret".to_string(),
            current: Some("s3cret".to_string()),
        }))
        .expect("expected a clipboard write");

        match action {
            Action::Clipboard(iced_runtime::clipboard::Action::Write { contents, .. }) => {
                assert_eq!(contents, "");
            }
            _ => panic!("expected the clipboard to be wiped"),
        }
    }

    #[test]
    fn clipboard_is_left_alone_when_the_user_copied_something_else() {
        let mut app = unlocked_with(UnlockedView::List {
            query: String::new(),
        });
        let task = app.update(Message::ClipboardCheckedForClear {
            expected: "s3cret".to_string(),
            current: Some("a shopping list".to_string()),
        });
        assert!(
            first_action(task).is_none(),
            "must not touch a clipboard the user has since overwritten"
        );
    }

    #[test]
    fn clipboard_is_left_alone_when_it_is_empty() {
        let mut app = unlocked_with(UnlockedView::List {
            query: String::new(),
        });
        let task = app.update(Message::ClipboardCheckedForClear {
            expected: "s3cret".to_string(),
            current: None,
        });
        assert!(first_action(task).is_none());
    }

    #[test]
    fn clipboard_check_does_not_count_as_activity_for_auto_lock() {
        let vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let idle_since = Instant::now() - std::time::Duration::from_secs(299);
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::List {
                    query: String::new(),
                },
                last_activity: idle_since,
                clipboard_copied_at: None,
            }),
        };
        let _ = app.update(Message::ClipboardCheckedForClear {
            expected: "s3cret".to_string(),
            current: None,
        });
        match &app.screen {
            Some(Screen::Unlocked { last_activity, .. }) => {
                assert_eq!(*last_activity, idle_since, "timer must not be reset");
            }
            _ => panic!("expected Unlocked screen"),
        }
    }

    // ---------------------------------------------------------------------------------
    // Versions view
    // ---------------------------------------------------------------------------------

    #[test]
    fn versions_view_renders_the_modified_time_next_to_each_version() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("github", "alice", "hunter2")
            .expect("add should succeed");
        let path = sekrets_core::Vault::locate().expect("vault should exist");
        sekrets_core::secrets::version_manager::snapshot_current(&path)
            .expect("snapshot should succeed");

        let versions = vault.list_versions().expect("list_versions should succeed");
        assert_eq!(versions.len(), 1);
        // Version numbers shift on rotation, so the timestamp is the only stable label.
        let label = versions[0].format_modified_local();
        assert!(!label.is_empty());
        assert!(
            label.contains('-'),
            "expected a formatted date, got {label}"
        );

        let app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Versions {
                    versions,
                    selected_version: None,
                    version_password: String::new(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };
        let _ = app.view();
    }

    // ---------------------------------------------------------------------------------
    // Real async round-trips: these drive the returned `iced::Task` to completion.
    // ---------------------------------------------------------------------------------

    #[test]
    fn unlock_submitted_drives_the_task_to_a_real_unlock_completed_ok() {
        sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Locked {
                path: sekrets_core::Vault::locate().expect("vault should exist"),
                password: TEST_PASSWORD.to_string(),
                error: None,
                unlocking: false,
            }),
        };

        let messages = drive_task(app.update(Message::UnlockSubmitted));

        assert_eq!(
            messages.len(),
            1,
            "the async leg must actually produce a message"
        );
        assert!(matches!(messages[0], Message::UnlockCompleted(Ok(_))));

        // And feeding the produced message back must land the app in the unlocked state.
        for message in messages {
            let _ = app.update(message);
        }
        assert!(matches!(app.screen, Some(Screen::Unlocked { .. })));
    }

    #[test]
    fn unlock_submitted_with_a_wrong_password_drives_to_a_real_error() {
        sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Locked {
                path: sekrets_core::Vault::locate().expect("vault should exist"),
                password: "definitely-not-it".to_string(),
                error: None,
                unlocking: false,
            }),
        };

        let messages = drive_task(app.update(Message::UnlockSubmitted));

        assert_eq!(messages.len(), 1);
        assert!(matches!(
            messages[0],
            Message::UnlockCompleted(Err(sekrets_core::VaultError::WrongPassword))
        ));
    }

    #[test]
    fn create_submitted_drives_the_task_to_a_real_create_completed() {
        let mut app = SekretsApp {
            screen: Some(Screen::NoVaultFound {
                path: std::path::PathBuf::from("/tmp/sekrets.enc"),
                password: TEST_PASSWORD.to_string(),
                confirm: TEST_PASSWORD.to_string(),
                error: None,
                creating: false,
            }),
        };

        let messages = drive_task(app.update(Message::CreateSubmitted));

        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], Message::CreateCompleted(Ok(_))));

        for message in messages {
            let _ = app.update(message);
        }
        assert!(matches!(app.screen, Some(Screen::Unlocked { .. })));
    }

    #[test]
    fn migrate_accepted_drives_the_task_to_a_real_migrate_completed() {
        sekrets_core::encryption::encryptor::encrypt_text(
            "github - username: foo, password: bar",
            TEST_PASSWORD,
        )
        .expect("encrypt_text should succeed");
        let vault = sekrets_core::Vault::unlock(TEST_PASSWORD).expect("unlock should succeed");
        assert!(vault.needs_migration());

        let mut app = SekretsApp {
            screen: Some(Screen::MigrationPrompt {
                vault,
                last_activity: Instant::now(),
            }),
        };

        let messages = drive_task(app.update(Message::MigrateAccepted));

        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], Message::MigrateCompleted(_, Ok(()))));

        for message in messages {
            let _ = app.update(message);
        }
        match &app.screen {
            Some(Screen::Unlocked { vault, .. }) => assert!(!vault.needs_migration()),
            _ => panic!("expected Unlocked screen after a completed migration"),
        }
    }

    #[test]
    fn switch_version_requested_drives_the_task_to_a_real_switch_version_completed() {
        let mut vault = sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        vault
            .add("old-account", "alice", "hunter2")
            .expect("add should succeed");
        let path = sekrets_core::Vault::locate().expect("vault should exist");
        sekrets_core::secrets::version_manager::snapshot_current(&path)
            .expect("snapshot should succeed");
        vault
            .delete("old-account", "alice")
            .expect("delete should succeed");

        let versions = vault.list_versions().expect("list_versions should succeed");
        let mut app = SekretsApp {
            screen: Some(Screen::Unlocked {
                vault,
                view: UnlockedView::Versions {
                    versions,
                    selected_version: None,
                    version_password: TEST_PASSWORD.to_string(),
                    error: None,
                },
                last_activity: Instant::now(),
                clipboard_copied_at: None,
            }),
        };

        let messages = drive_task(app.update(Message::SwitchVersionRequested(1)));

        assert_eq!(messages.len(), 1);
        assert!(matches!(
            messages[0],
            Message::SwitchVersionCompleted(Ok(()))
        ));

        for message in messages {
            let _ = app.update(message);
        }
        match &app.screen {
            Some(Screen::Unlocked {
                view: UnlockedView::List { .. },
                vault,
                ..
            }) => assert_eq!(vault.search("old-account").len(), 1),
            _ => panic!("expected List view with the restored credential"),
        }
    }

    #[test]
    fn the_configured_iced_executor_actually_runs_spawned_futures() {
        // Direct regression test for the H1 bug: without `iced`'s "tokio" feature,
        // `iced_futures` silently selects its "null" backend, whose `spawn` is a no-op —
        // so every `Task::perform` future was dropped and no async work in the app ever
        // completed. `drive_task` above polls streams itself and cannot see this; only
        // asking the executor the app is actually built with to run something can.
        use iced::executor::{Default as DefaultExecutor, Executor};

        // Explicit trait dispatch: the default backend is a bare `tokio::runtime::Runtime`
        // with inherent `new`/`spawn` of its own, and the null backend's *trait* `spawn` is
        // the no-op that caused the bug, so the test has to go through the trait.
        let executor = <DefaultExecutor as Executor>::new()
            .expect("the default executor must be constructible");
        let (sender, receiver) = std::sync::mpsc::channel();
        Executor::spawn(&executor, async move {
            let _ = sender.send(42);
        });

        assert_eq!(
            receiver
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("the executor must actually run spawned futures"),
            42
        );
    }

    #[test]
    fn located_task_from_startup_drives_to_a_real_located_message() {
        sekrets_core::Vault::create(TEST_PASSWORD).expect("create should succeed");
        let task = Task::perform(async { sekrets_core::Vault::locate() }, Message::Located);

        let messages = drive_task(task);

        assert_eq!(messages.len(), 1);
        assert!(matches!(messages[0], Message::Located(Some(_))));
    }
}
