use super::{
    error::KoraError,
    signer::{Signature, Signer},
};
use solana_sdk::{signature::Keypair, signer::Signer as SolanaSigner};

/// A Solana-based signer that uses an in-memory keypair
pub struct SolanaMemorySigner {
    keypair: Keypair,
}

impl SolanaMemorySigner {
    /// Creates a new signer from a Solana keypair
    pub fn new(keypair: Keypair) -> Self {
        Self { keypair }
    }

    /// Creates a new signer from a private key byte array
    pub fn from_bytes(private_key: &[u8]) -> Result<Self, KoraError> {
        let keypair = Keypair::from_bytes(private_key)
            .map_err(|e| KoraError::Signer(format!("Invalid private key bytes: {}", e)))?;
        Ok(Self { keypair })
    }

    /// Creates a new signer from a base58-encoded private key string
    pub fn from_base58(private_key: &str) -> Result<Self, KoraError> {
        let keypair = Keypair::from_base58_string(private_key);
        Ok(Self { keypair })
    }

    /// Get the public key of this signer
    pub fn pubkey(&self) -> [u8; 32] {
        self.keypair.pubkey().to_bytes()
    }

    /// Get the base58-encoded public key
    pub fn pubkey_base58(&self) -> String {
        bs58::encode(self.pubkey()).into_string()
    }
}

impl Clone for SolanaMemorySigner {
    fn clone(&self) -> Self {
        Self::from_bytes(&self.keypair.to_bytes()).expect("Failed to clone keypair")
    }
}

impl Signer for SolanaMemorySigner {
    type Error = KoraError;

    fn partial_sign(&self, message: &[u8]) -> Result<Signature, Self::Error> {
        self.full_sign(message)
    }

    fn full_sign(&self, message: &[u8]) -> Result<Signature, Self::Error> {
        let solana_sig = self.keypair.sign_message(message);
        Ok(Signature { bytes: solana_sig.as_ref().to_vec(), is_partial: false })
    }
}
