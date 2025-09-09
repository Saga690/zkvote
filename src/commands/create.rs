use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use rand::RngCore;

#[derive(Serialize, Deserialize)]
struct Identity {
    trapdoor: u128,
    nullifier: u128,
}

#[derive(Serialize, Deserialize)]
struct Proposal {
    id: String,
    question: String,
    options: Vec<String>,
    identity_commitments: Vec<String>,
    created_at: String,
    voters: Vec<String>,
}

fn load_identity(path: PathBuf) -> anyhow::Result<Identity> {
    let data = fs::read_to_string(path)?;
    let id: Identity = serde_json::from_str(&data)?;
    Ok(id)
}

fn commitment_of(id: &Identity) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.trapdoor.to_le_bytes());
    hasher.update(id.nullifier.to_le_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

fn generate_hex_id() -> String {
    let mut bytes = [0u8; 8]; // 8 bytes = 16 hex chars
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub async fn handle_create(question: String) {
    // 1) load local identity
    let id_path = PathBuf::from("identity.json");
    let identity = match load_identity(id_path) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Could not load identity.json. Did you run `cargo run -- register`?\nError: {e}");
            return;
        }
    };

    // 2) compute commitment and hex id
    let me = commitment_of(&identity);
    let hex_id = generate_hex_id();

    // 3) build proposal object
    let proposal = Proposal {
        id: hex_id.clone(),
        question: question.clone(),
        options: vec!["yes".to_string(), "no".to_string()],
        identity_commitments: vec![me],
        created_at: Utc::now().to_rfc3339(),
        voters: vec![],
    };

    // 4) write proposals/<slug>.json
    let out_path = PathBuf::from("proposals").join(format!("{hex_id}.json"));

    if let Err(e) = fs::write(&out_path, serde_json::to_string_pretty(&proposal).unwrap()) {
        eprintln!("Failed to write proposal file: {e}");
        return;
    }

    println!("Proposal created");
    println!("File: {}", out_path.display());
}
