use russh::ChannelId;
use russh::CryptoVec;
use russh::server::Session;

pub fn send_message(
    user: crate::users::User,
    msg: String,
    server: &mut crate::server::Server,
    channel: ChannelId,
    session: &mut Session,
) {
    let data = CryptoVec::from(format!("\r[{}] {}\r\n", user.clone().name(), msg));

    let handle = tokio::runtime::Handle::current();
    let guard = handle.enter();
    futures::executor::block_on(server.post(data.clone()));
    drop(guard);

    session.data(channel, data).unwrap();

    let name_data = CryptoVec::from(format!("[{}] ", user.name()).as_bytes().to_vec());
    session.data(channel, name_data).unwrap();
}

pub async fn send_message_async(
    user: crate::users::User,
    msg: String,
    server: &mut crate::server::Server,
    channel: ChannelId,
    session: &mut Session,
) {
    let data = CryptoVec::from(format!("\r[{}] {}\r\n", user.clone().name(), msg));

    server.post(data.clone()).await;

    session.data(channel, data).unwrap();

    let name_data = CryptoVec::from(format!("[{}] ", user.name()).as_bytes().to_vec());
    session.data(channel, name_data).unwrap();
}

pub fn send_message_no_self(
    user: crate::users::User,
    msg: String,
    server: &mut crate::server::Server,
) {
    let data = CryptoVec::from(format!("\r[{}] {}\r\n", user.name(), msg));

    let handle = tokio::runtime::Handle::current();
    let guard = handle.enter();
    futures::executor::block_on(server.post(data.clone()));
    drop(guard);
}

pub async fn send_message_async_no_self(
    user: crate::users::User,
    msg: String,
    server: &mut crate::server::Server,
) {
    let data = CryptoVec::from(format!("\r[{}] {}\r\n", user.name(), msg));

    server.post(data.clone()).await;
}
