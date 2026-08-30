use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT, ristretto::CompressedRistretto, scalar::Scalar,
};
use rand_core::OsRng;
use sha2::{Digest, Sha512};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchnorrProof {
    pub public_key: CompressedRistretto,
    pub commitment: CompressedRistretto,
    pub response: Scalar,
}

fn challenge(
    public_key: &CompressedRistretto,
    commitment: &CompressedRistretto,
    statement: &[u8],
) -> Scalar {
    let mut h = Sha512::new();
    h.update(b"FinLang/Schnorr/FS/v1");
    h.update(public_key.as_bytes());
    h.update(commitment.as_bytes());
    h.update(statement);
    Scalar::from_hash(h)
}

pub fn generate_keypair() -> (Scalar, CompressedRistretto) {
    let secret = Scalar::random(&mut OsRng);
    let public = (RISTRETTO_BASEPOINT_POINT * secret).compress();
    (secret, public)
}

pub fn prove(secret: &Scalar, statement: &[u8]) -> SchnorrProof {
    let public = (RISTRETTO_BASEPOINT_POINT * *secret).compress();
    let nonce = Scalar::random(&mut OsRng);
    let commitment = (RISTRETTO_BASEPOINT_POINT * nonce).compress();
    let c = challenge(&public, &commitment, statement);
    SchnorrProof {
        public_key: public,
        commitment,
        response: nonce + c * secret,
    }
}

pub fn verify(proof: &SchnorrProof, statement: &[u8]) -> bool {
    let Some(public) = proof.public_key.decompress() else {
        return false;
    };
    let Some(commitment) = proof.commitment.decompress() else {
        return false;
    };
    let c = challenge(&proof.public_key, &proof.commitment, statement);
    RISTRETTO_BASEPOINT_POINT * proof.response == commitment + public * c
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_proof_verifies() {
        let (s, _) = generate_keypair();
        let p = prove(&s, b"statement");
        assert!(verify(&p, b"statement"));
    }
    #[test]
    fn modified_statement_fails() {
        let (s, _) = generate_keypair();
        let p = prove(&s, b"statement");
        assert!(!verify(&p, b"changed"));
    }
    #[test]
    fn modified_response_fails() {
        let (s, _) = generate_keypair();
        let mut p = prove(&s, b"statement");
        p.response += Scalar::ONE;
        assert!(!verify(&p, b"statement"));
    }
}
