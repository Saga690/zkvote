use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Deserialize)]
struct VoteRecord {
    // commitment: String,
    choice: String,
}

pub async fn handle_tally(proposal_id: String) {
    // Votes are stored in votes/{proposal_id}.json
    let vote_path = PathBuf::from("votes").join(format!("{proposal_id}.json"));

    if !vote_path.exists() {
        eprintln!("No votes found for proposal {proposal_id}");
        return;
    }

    // Load votes
    let vdata = fs::read_to_string(&vote_path).expect("Failed to read votes file.");
    let votes: Vec<VoteRecord> = serde_json::from_str(&vdata).unwrap_or_default();

    if votes.is_empty() {
        println!("No votes have been cast yet for proposal {proposal_id}");
        return;
    }

    // Count per choice
    let mut counts: HashMap<String, usize> = HashMap::new();
    for v in &votes {
        *counts.entry(v.choice.clone()).or_insert(0) += 1;
    }

    // Print results
    println!("Results for Proposal {proposal_id}:");
    for (choice, count) in counts {
        println!("  {choice}: {count}");
    }
    println!("  Total votes: {}", votes.len());
}
