use std::path::PathBuf;

/// queer.toml file
#[derive(Clone)]
pub struct Config {
    /// Path to the users.toml file.
    pub path_to_users: PathBuf
}