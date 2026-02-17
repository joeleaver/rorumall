use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key: String,
    pub key_id: Option<String>,
}

const STORAGE_KEY: &str = "ofscp_client_keys";

pub fn generate_keypair() -> KeyPair {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    KeyPair {
        public_key: BASE64.encode(verifying_key.as_bytes()),
        private_key: BASE64.encode(signing_key.to_bytes()),
        key_id: None,
    }
}

pub fn save_keypair(keys: &KeyPair) {
    crate::storage::save(STORAGE_KEY, keys);
}

pub fn load_keypair() -> Option<KeyPair> {
    crate::storage::load(STORAGE_KEY)
}

pub fn clear_keypair() {
    crate::storage::remove(STORAGE_KEY);
}

#[derive(Clone, Debug)]
pub struct SignedHeaders {
    pub actor: String,
    pub key_id: String,
    pub timestamp: String,
    pub signature: String,
}

pub fn sign_request(
    method: &str,
    path: &str,
    body: &[u8],
    keys: &KeyPair,
    handle: &str,
    domain: &str,
) -> Option<SignedHeaders> {
    let key_id = keys.key_id.as_ref()?;

    let priv_bytes = BASE64.decode(&keys.private_key).ok()?;
    let priv_arr: [u8; 32] = priv_bytes.try_into().ok()?;
    let signing_key = SigningKey::from_bytes(&priv_arr);

    let timestamp = Utc::now().to_rfc3339();
    let body_hash = hex::encode(Sha256::digest(body));
    let canonical = format!("{}\n{}\n{}\n{}", method, path, timestamp, body_hash);

    let signature = signing_key.sign(canonical.as_bytes());
    let sig_b64 = BASE64.encode(signature.to_bytes());

    Some(SignedHeaders {
        actor: format!("@{}@{}", handle, domain),
        key_id: key_id.clone(),
        timestamp,
        signature: sig_b64,
    })
}

#[derive(Clone, Debug)]
pub struct WsAuthParams {
    pub actor: String,
    pub key_id: String,
    pub timestamp: String,
    pub signature: String,
}

impl WsAuthParams {
    pub fn to_query_string(&self) -> String {
        format!(
            "actor={}&timestamp={}&keyId={}&signature={}",
            urlencoding::encode(&self.actor),
            urlencoding::encode(&self.timestamp),
            urlencoding::encode(&self.key_id),
            urlencoding::encode(&self.signature)
        )
    }
}

pub fn sign_ws_request(
    path: &str,
    keys: &KeyPair,
    handle: &str,
    domain: &str,
) -> Option<WsAuthParams> {
    let key_id = keys.key_id.as_ref()?;

    let priv_bytes = BASE64.decode(&keys.private_key).ok()?;
    let priv_arr: [u8; 32] = priv_bytes.try_into().ok()?;
    let signing_key = SigningKey::from_bytes(&priv_arr);

    let timestamp = Utc::now().to_rfc3339();
    let body_hash = hex::encode(Sha256::digest([]));
    let canonical = format!("GET\n{}\n{}\n{}", path, timestamp, body_hash);

    let signature = signing_key.sign(canonical.as_bytes());
    let sig_b64 = BASE64.encode(signature.to_bytes());

    Some(WsAuthParams {
        actor: format!("@{}@{}", handle, domain),
        key_id: key_id.clone(),
        timestamp,
        signature: sig_b64,
    })
}
