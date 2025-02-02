use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use russh::keys::{Certificate, *};
use russh::server::{Msg, Session};
use russh::*;

use crate::messages::{send_message, send_message_async, send_message_no_self};
use crate::users::USER_QUEERBOT;

#[derive(Clone)]
pub struct Server {
    pub clients: Arc<
        tokio::sync::Mutex<
            HashMap<
                usize,
                (
                    ChannelId,
                    russh::server::Handle,
                    Arc<Mutex<crate::users::User>>,
                    Arc<Mutex<Vec<u8>>>,
                ),
            >,
        >,
    >,
    pub id: usize,
    pub user: Arc<Mutex<crate::users::User>>,
    pub cfg: crate::config::Config,
    pub buf: Arc<Mutex<Vec<u8>>>,
}

impl Server {
    pub async fn post(&mut self, data: CryptoVec) {
        let mut clients = self.clients.lock().await;
        for (id, (channel, s, user, buf)) in clients.iter_mut() {
            if *id != self.id {
                let _ = s.data(*channel, data.clone()).await;

                let raw_data = vec![
                    format!("[{}] ", user.lock().unwrap().clone().name())
                        .as_bytes()
                        .to_vec(),
                    buf.lock().unwrap().clone(),
                ]
                .concat();

                let name_data = CryptoVec::from(raw_data);
                let _ = s.data(*channel, name_data).await;
            }
        }
    }
}

impl server::Server for Server {
    type Handler = Self;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let mut s = self.clone();
        s.user = Arc::new(Mutex::new(USER_QUEERBOT.clone()));
        s.user.lock().unwrap().id = s.id;
        s.user.lock().unwrap().name = Some(format!("Unknown{}", s.id));
        s.user.lock().unwrap().key = None;

        s.buf = Arc::new(Mutex::new(vec![]));
        self.id += 1;
        s
    }
    fn handle_session_error(&mut self, error: <Self::Handler as russh::server::Handler>::Error) {
        eprintln!("Session error: {:#?}", error);
    }
}

impl server::Handler for Server {
    type Error = russh::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        {
            let mut clients = self.clients.lock().await;
            clients.insert(
                self.id,
                (
                    channel.id(),
                    session.handle(),
                    self.user.clone(),
                    self.buf.clone(),
                ),
            );
        }
        if self.cfg.motd.is_some() {
            if self.cfg.motd.as_ref().unwrap().login_message.is_some() {
                let data = CryptoVec::from(
                    self.cfg
                        .motd
                        .as_ref()
                        .unwrap()
                        .login_message
                        .as_ref()
                        .unwrap()
                        .clone(),
                );
                session.data(channel.id(), data)?;
            }
        }
        send_message(
            crate::users::USER_QUEERBOT,
            format!("@{} has joined!", self.user.lock().unwrap().clone().name()),
            self,
            channel.id(),
            session,
        );
        Ok(true)
    }

    async fn auth_publickey(
        &mut self,
        name: &str,
        key: &ssh_key::PublicKey,
    ) -> Result<server::Auth, Self::Error> {
        self.user.lock().unwrap().key = Some((*key).clone());
        self.user.lock().unwrap().name = Some(name.to_owned());
        Ok(server::Auth::Accept)
    }

    async fn auth_openssh_certificate(
        &mut self,
        _name: &str,
        _certificate: &Certificate,
    ) -> Result<server::Auth, Self::Error> {
        Ok(server::Auth::UnsupportedMethod)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Sending Ctrl+C ends the session and disconnects the client
        if data == [3] {
            return Err(russh::Error::Disconnect);
        }

        if self.buf.lock().unwrap().len() > self.cfg.max_msg_len.unwrap_or(500) && data != b"\n" {
            let dcrypto = CryptoVec::from("\x07");
            session.data(channel, dcrypto)?;
            return Ok(());
        }

        let dcrypto = CryptoVec::from(data); // echo back incoming text
        session.data(channel, dcrypto)?;

        if data.contains(&8) {
            self.buf.lock().unwrap().pop();
            return Ok(());
        }
        self.buf.lock().unwrap().append(&mut data.to_vec());

        println!("aaaa {:x?}", self.buf.lock().unwrap());

        if data.contains(&0xd) {
            println!("trying to send");
            let s = String::from_utf8(self.buf.lock().unwrap().clone());

            if s.is_err() {
                send_message_async(
                    crate::users::USER_QUEERBOT,
                    format!(
                        "Hey everyone! @{} just tried to send invalid UTF8! Don't worry, I spared you.",
                        self.user.lock().unwrap().clone().name()
                    ),
                    &mut self.clone(),
                    channel,
                    session
                ).await;
                return Ok(());
            }

            self.buf.lock().unwrap().clear();

            let user = self.user.lock().unwrap().clone();

            send_message_async(user, s.unwrap(), &mut self.clone(), channel, session).await;
        }
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        send_message_no_self(
            crate::users::USER_QUEERBOT,
            format!("@{} has left!", self.user.lock().unwrap().clone().name()),
            self,
        );
        let id = self.id;
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let mut clients = clients.lock().await;
            clients.remove(&id);
        });
    }
}
