use russh::keys::ssh_key;

/// A struct used for storing user data.
#[derive(Clone)]
pub struct User {
    /// The ID of the server instance that is used.
    pub id: usize,
    /// The name of the user.
    pub name: Option<String>,
    /// The SSH key that the user connected with. Should ONLY be None in the case of Queerbot.
    pub key: Option<ssh_key::PublicKey>
}

impl User {
    pub fn name(self) -> String {
        let mut name = self.name.unwrap_or("Queerbot".to_string());
        name.remove_matches(|c: char| c.is_whitespace());
        if self.id == 0 {
            return name;
        }
        format!("{}#{}", name, self.id)
    }
}

/// The queerbot user. Used internally for server actions.
pub const USER_QUEERBOT: User = User {
    id: 0,
    name: None,
    key: None,
};
