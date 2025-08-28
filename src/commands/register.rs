use rand::Rng;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Identity {
    pub trapdoor: u128,
    pub nullifier: u128,
}

impl Identity {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        Identity {
            trapdoor: rng.r#gen(),
            nullifier: rng.r#gen(),
        }
    }

    pub fn commitment(&self) -> String {        // commitment = hash(trapdoor + nullifier), this is a merkle leaf in the identity tree
        let mut hasher = Sha256::new();
        hasher.update(self.trapdoor.to_le_bytes());
        hasher.update(self.nullifier.to_le_bytes());
        let result = hasher.finalize();     // 32 byte final hash
        hex::encode(result)     // composed of trapdoor + nullifier
    }

    pub fn save_to_file(&self, path: PathBuf) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        fs::write(path, json).expect("Failed to write identity to file");
    }
}

pub async fn handle_register() {
    let identity = Identity::generate();
    let path = PathBuf::from("identity.json");

    identity.save_to_file(path);
    let commitment = identity.commitment();

    println!("Identity registered!");
    println!("Trapdoor + Nullifier saved locally.");
    println!("Public Commitment: {}", commitment);
}

//Future plan: Add Merkle Tree