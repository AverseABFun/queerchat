#![feature(duration_constructors)]

use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::{fs, io};

use base64::prelude::*;
use russh::keys::{decode_openssh, PrivateKey};
use russh::server::Server as _;
use russh::*;
use tokio::sync::Mutex;
use users::USER_QUEERBOT;

mod server;
mod users;
mod config;

use crate::server::*;

#[tokio::main]
async fn main() {
    println!("Queerchat v{} starting...", env!("CARGO_PKG_VERSION"));

    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    #[allow(deprecated)]
    let search_path = fs::read_dir(std::env::home_dir().unwrap().join(".ssh")).unwrap();
    let mut priv_key_paths: Vec<String> = vec![];
    let mut priv_keys: Vec<Vec<u8>> = vec![];
    for entry in search_path {
        if entry.as_ref().unwrap().file_type().unwrap().is_dir() {
            continue;
        }
        if !entry
            .as_ref()
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .starts_with("id_")
        {
            continue;
        }
        if entry
            .as_ref()
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .ends_with(".pub")
        {
            continue;
        }
        let data = fs::read(entry.as_ref().unwrap().path()).unwrap();
        priv_key_paths.push(entry.as_ref().unwrap().path().to_str().unwrap().to_owned());
        priv_keys.push(data);
    }

    let mut key: &[u8] = &[];

    if priv_keys.len() > 1 {
        let mut i = 1usize;
        for path in priv_key_paths {
            println!("{}: {}", i, path);
            i += 1;
        }
        print!("Which key do you want to use (1-{})? ", i - 1);
        io::stdout().flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let mut f = str::parse::<usize>(buffer.as_str().trim());
        while f.as_ref().is_err() || *f.as_ref().unwrap() > i {
            if f.is_err() {
                println!("That's not a positive number!");
            } else {
                println!("That's too big a number!")
            }

            print!("Which key do you want to use (1-{})? ", i - 1);
            io::stdout().flush().unwrap();

            io::stdin().read_line(&mut buffer).unwrap();
            f = str::parse::<usize>(buffer.as_str().trim());
        }
        key = priv_keys[f.unwrap() - 1].as_slice();
    }

    let mut keystr = std::str::from_utf8(key).unwrap().trim();
    if !keystr.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----\n")
        || !keystr.ends_with("-----END OPENSSH PRIVATE KEY-----")
    {
        println!("Key is not valid private key");
        return;
    }
    let binding = keystr
        .replace("-----BEGIN OPENSSH PRIVATE KEY-----\n", "")
        .replace("-----END OPENSSH PRIVATE KEY-----", "")
        .replace("\n", "");
    keystr = binding.as_str();
    let binding = BASE64_STANDARD.decode(keystr).unwrap();
    key = binding.as_slice();

    let pk = PrivateKey::from_bytes(key).unwrap();
    let priv_key: PrivateKey;
    if pk.is_encrypted() {
        print!("This key is encrypted; enter the password(text will not show): ");
        let pwd = rpassword::read_password().unwrap();
        priv_key = decode_openssh(key, Some(pwd.as_str())).unwrap();
    } else {
        priv_key = decode_openssh(key, None).unwrap();
    }

    let mut methods = MethodSet::empty();
    methods.push(MethodKind::PublicKey);

    let config = russh::server::Config {
        inactivity_timeout: None,
        auth_rejection_time: std::time::Duration::from_mins(45),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![priv_key],
        preferred: Preferred {
            // kex: std::borrow::Cow::Owned(vec![russh::kex::DH_GEX_SHA256]),
            ..Preferred::default()
        },
        methods,
        ..Default::default()
    };
    let config = Arc::new(config);

    let mut sh = Server {
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 1,
        user: USER_QUEERBOT
    };

    sh.run_on_address(config, ("0.0.0.0", 8184)).await.unwrap();
}