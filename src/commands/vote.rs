use crate::commands::register::{Identity, Proposal}; 
use std::{fs, path::PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct VoteRecord {
    commitment: String,
    choice: String,
}

pub async fn handle_vote(proposal_id: String, choice: String) {
    // 1. Load identity
    let identity = Identity::load_from_file(PathBuf::from("identity.json"))
        .expect("No identity.json found. Run `register` first.");
    let commitment = identity.commitment();

    // 2. Load proposal
    let path = PathBuf::from("proposals").join(format!("{proposal_id}.json"));
    let data = fs::read_to_string(&path).expect("Proposal file not found.");
    let proposal: Proposal = serde_json::from_str(&data).unwrap();

    // 3. Ensure voter is registered
    if !proposal.voters.contains(&commitment) {
        eprintln!("You are not registered for this proposal. Run `register-to-proposal` first.");
        return;
    }

    // 4. Save vote into a separate file (to keep votes distinct)
    let vote_path = PathBuf::from("votes").join(format!("{proposal_id}.json"));
    let mut votes: Vec<VoteRecord> = if vote_path.exists() {
        let vdata = fs::read_to_string(&vote_path).unwrap();
        serde_json::from_str(&vdata).unwrap_or_default()
    } else {
        vec![]
    };

    // Check if already voted
    if votes.iter().any(|v| v.commitment == commitment) {
        println!("You already voted on this proposal.");
        return;
    }

    // Append new vote
    votes.push(VoteRecord {
        commitment,
        choice: choice.clone(),
    });

    fs::write(&vote_path, serde_json::to_string_pretty(&votes).unwrap())
        .expect("Failed to record vote.");

    println!("Vote recorded: {}", choice);
}
