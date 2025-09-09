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

    pub fn load_from_file(path: PathBuf) -> Option<Self> {
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Proposal {
    pub question: String,
    pub options: Vec<String>,
    pub voters: Vec<String>,
}

pub async fn handle_register_identity() {
    let identity = Identity::generate();
    let path = PathBuf::from("identity.json");

    identity.save_to_file(path);
    let commitment = identity.commitment();

    println!("Identity registered!");
    println!("Trapdoor + Nullifier saved locally.");
    println!("Public Commitment: {}", commitment);
}

//Future plan: Add Merkle Tree

pub async fn handle_register_to_proposal(slug: &str) {
    // Load identity
    let identity = Identity::load_from_file(PathBuf::from("identity.json"))
        .expect("No identity.json found. Run `register` first.");
    let commitment = identity.commitment();

    // Load proposal
    let path = PathBuf::from("proposals").join(format!("{slug}.json"));
    let data = fs::read_to_string(&path).expect("Proposal file not found.");
    let mut proposal: Proposal = serde_json::from_str(&data).unwrap();

    // Add voter if not already registered
    if !proposal.voters.contains(&commitment) {
        proposal.voters.push(commitment.clone());
        fs::write(&path, serde_json::to_string_pretty(&proposal).unwrap())
            .expect("Failed to update proposal file.");
        println!("Registered commitment {} to proposal {}", commitment, slug);
    } else {
        println!("Already registered for proposal {}", slug);
    }
}