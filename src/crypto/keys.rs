use crate::bytes::{AsBytes, FromBytes};
use ssb_crypto::ephemeral::{EphPublicKey, EphSecretKey};
use ssb_crypto::{PublicKey, Signature};

/// Client's long-term public key.
#[derive(Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct ClientPublicKey(pub PublicKey);

/// Server's long-term public key; known to client prior to the handshake.
#[derive(Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct ServerPublicKey(pub PublicKey);

#[derive(Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct ClientSignature(pub Signature);

#[derive(Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct ServerSignature(pub Signature);

/// Client ephemeral public key (generated anew for each connection)
#[derive(Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct ClientEphPublicKey(pub EphPublicKey);

/// Client ephemeral secret key
pub struct ClientEphSecretKey(pub EphSecretKey);

/// Server ephemeral public key (generated anew for each connection)
#[derive(Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
pub struct ServerEphPublicKey(pub EphPublicKey);

/// Server ephemeral secret key
pub struct ServerEphSecretKey(pub EphSecretKey);
