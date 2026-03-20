#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

/// An SSH-agent-wrapped repo key.
///
/// The key is encrypted using a symmetric key derived from an SSH agent
/// signature over a random challenge. Only someone with access to the
/// corresponding private key via the SSH agent can reproduce the signature
/// and recover the repo key.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgentWrappedKey {
    /// Format version for forward compatibility.
    pub version: u32,
    /// SSH key fingerprint (e.g. `SHA256:...`) identifying the recipient.
    pub fingerprint: String,
    /// Base64-encoded 32-byte random challenge signed by the agent.
    pub challenge: String,
    /// Base64-encoded 12-byte nonce for `ChaCha20Poly1305`.
    pub nonce: String,
    /// Base64-encoded `ChaCha20Poly1305` ciphertext of the repo key.
    pub encrypted_repo_key: String,
}
