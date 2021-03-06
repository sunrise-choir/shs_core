use crate::bytes::{as_mut, as_ref};
use crate::crypto::outcome::HandshakeKeys;
use crate::crypto::{keys::*, message::*, outcome::*, shared_secret::*};
use crate::error::HandshakeError;
use crate::util::send;

use ssb_crypto::{ephemeral::generate_ephemeral_keypair, Keypair, NetworkKey, PublicKey};

use core::mem::size_of;
use futures_io::{AsyncRead, AsyncWrite};
use futures_util::io::{AsyncReadExt, AsyncWriteExt};
use std::io;

/// Perform the client side of the handshake over an `AsyncRead + AsyncWrite` stream.
/// Closes the stream on handshake failure.
pub async fn client_side<S>(
    mut stream: S,
    net_key: &NetworkKey,
    keypair: &Keypair,
    server_pk: &PublicKey,
) -> Result<HandshakeKeys, HandshakeError<io::Error>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let r = try_client_side(&mut stream, net_key, keypair, server_pk).await;
    if r.is_err() {
        stream.close().await.unwrap_or(());
    }
    r
}

async fn try_client_side<S>(
    mut stream: S,
    net_key: &NetworkKey,
    keypair: &Keypair,
    server_pk: &PublicKey,
) -> Result<HandshakeKeys, HandshakeError<io::Error>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    use HandshakeError::*;

    let server_pk = ServerPublicKey(*server_pk);
    let (eph_pk, eph_sk) = {
        let (p, s) = generate_ephemeral_keypair();
        (ClientEphPublicKey(p), ClientEphSecretKey(s))
    };

    send(&mut stream, ClientHello::new(&eph_pk, &net_key)).await?;

    let server_eph_pk = {
        let mut buf = [0u8; size_of::<ServerHello>()];
        stream.read_exact(&mut buf).await?;
        as_mut::<ServerHello>(&mut buf)
            .verify(&net_key)
            .ok_or(ServerHelloVerifyFailed)?
    };

    // Derive shared secrets
    let shared_a = SharedA::client_side(&eph_sk, &server_eph_pk).ok_or(SharedAInvalid)?;
    let shared_b = SharedB::client_side(&eph_sk, &server_pk).ok_or(SharedBInvalid)?;
    let shared_c = SharedC::client_side(&keypair, &server_eph_pk).ok_or(SharedCInvalid)?;

    // Send client auth
    send(
        &mut stream,
        ClientAuth::new(&keypair, &server_pk, &net_key, &shared_a, &shared_b),
    )
    .await?;

    let mut buf = [0u8; size_of::<ServerAccept>()];
    stream.read_exact(&mut buf).await?;
    as_ref::<ServerAccept>(&buf)
        .verify(
            &keypair, &server_pk, &net_key, &shared_a, &shared_b, &shared_c,
        )
        .ok_or(ServerAcceptVerifyFailed)?;

    Ok(HandshakeKeys {
        read_key: server_to_client_key(
            &ClientPublicKey(keypair.public),
            &net_key,
            &shared_a,
            &shared_b,
            &shared_c,
        ),
        read_starting_nonce: starting_nonce(&net_key, &eph_pk.0),

        write_key: client_to_server_key(&server_pk, &net_key, &shared_a, &shared_b, &shared_c),
        write_starting_nonce: starting_nonce(&net_key, &server_eph_pk.0),

        peer_key: server_pk.0,
    })
}
