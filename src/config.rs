#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct User {
    pub name: Option<String>,
    pub key: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct UserIP {
    pub name: Option<String>,
    pub key: Option<String>,
    pub ip: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct UserConfig {
    pub allowlist: Option<Vec<User>>,
    pub blacklist: Option<Vec<UserIP>>,
    pub mapping: Option<Vec<User>>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum MaybeUserConfig {
    No,
    Path(String),
    User(UserConfig),
}

impl Default for MaybeUserConfig {
    fn default() -> Self {
        MaybeUserConfig::No
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MotdMotd {
    pub is_file: bool,
    pub text: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MotdFailedLoginMessage {
    pub invalid_key: Option<String>,
    pub blacklisted: Option<String>,
    pub not_whitelisted: Option<String>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Motd {
    pub server_name: Option<String>,
    pub motd: Option<MotdMotd>,
    pub login_message: Option<String>,
    pub failed_login_message: Option<MotdFailedLoginMessage>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub users: Option<MaybeUserConfig>,
    pub prevent_spoofing: Option<bool>,
    pub motd: Option<Motd>,
    pub max_msg_len: Option<usize>,
}

pub fn deserialize_config(s: &str) -> Result<Config, toml::de::Error> {
    let mut out: Result<Config, toml::de::Error> = toml::de::from_str(s);
    if out.is_ok() {
        let mut d = out.unwrap();
        d.users = Some(MaybeUserConfig::Path("./queerusers.toml".to_string()));
        out = Ok(d);
    }
    out
}

pub fn serialize_config(s: &Config) -> Result<String, toml::ser::Error> {
    toml::ser::to_string(s)
}
