use russh::keys::ssh_key;

/// A struct used for storing user data.
#[derive(Clone)]
pub struct User<'a> {
    /// The ID of the server instance that is used.
    pub id: usize,
    /// The name of the user.
    pub name: &'a str,
    /// The SSH key that the user connected with. Should ONLY be None in the case of Queerbot.
    pub key: Option<ssh_key::PublicKey>,
}

/// The queerbot user. Used internally for server actions.
pub const USER_QUEERBOT: User = User {
    id: 0,
    name: "Queerbot",
    key: None,
};