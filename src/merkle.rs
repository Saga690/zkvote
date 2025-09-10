// Let's trace the process with four leaves: L1, L2, L3, L4.
//     Building the Tree:
//         Level 0 (Leaves): [H(L1), H(L2), H(L3), H(L4)]
//         Level 1: [H(H(L1)||H(L2)), H(H(L3)||H(L4))]
//         Level 2 (Root): [H(H(H(L1)||H(L2))||H(H(L3)||H(L4)))]
//     Generating a Proof for L3 (index 2):
//         Level 0: The current index is 2. Its sibling is H(L4) at index 3. The sibling is on the right (sibling_is_left = false). The proof now has { sibling_hashes: [H(L4)], sibling_is_left: [false] }. The parent index is 2 / 2 = 1.
//         Level 1: The current index is 1. Its sibling is H(H(L1)||H(L2)) at index 0. The sibling is on the left (sibling_is_left = true). The proof is now { sibling_hashes: [H(L4), H(H(L1)||H(L2))], sibling_is_left: [false, true] }. The parent index is 1 / 2 = 0.
//         The loop terminates as we've reached the root level.
//     Verifying the Proof for L3:
//         Start with current_hex = H(L3).
//         Step 1: The first sibling is H(L4) and sibling_is_left is false. So, calculate parent1 = combine_hex(current_hex, H(L4)), which is H(H(L3)||H(L4)). Update current_hex to parent1.
//         Step 2: The next sibling is H(H(L1)||H(L2)) and sibling_is_left is true. So, calculate parent2 = combine_hex(H(H(L1)||H(L2)), current_hex). This is H(H(H(L1)||H(L2))||H(H(L3)||H(L4))).
//         Compare this final calculated hash with the tree's root hash. They will match.

use sha2::{Digest, Sha256};
use hex;
use std::fmt;

#[derive(Debug, Clone)]
pub struct MerkleTree {
    // levels[0] = leaves (hex strings), levels[last] = root (single element)
    pub levels: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// sibling hashes (hex) at each level, bottom-up
    pub sibling_hashes: Vec<String>,
    /// sibling_is_left[i] == true means sibling at that level was the left node
    pub sibling_is_left: Vec<bool>,
}

impl fmt::Display for MerkleProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, h) in self.sibling_hashes.iter().enumerate() {
            writeln!(f, "level {}: sibling_is_left={} hash={}", i, self.sibling_is_left[i], h)?;
        }
        Ok(())
    }
}

fn hash_pair(left: &[u8], right: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let res = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&res);
    arr
}

/// Helper: hex string -> bytes
fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    hex::decode(s).ok()
}

/// Create SHA256(hexdecode(left) || hexdecode(right)) and return hex string
fn combine_hex(left_hex: &str, right_hex: &str) -> Option<String> {
    let l = hex_to_bytes(left_hex)?;
    let r = hex_to_bytes(right_hex)?;
    let h = hash_pair(&l, &r);
    Some(hex::encode(h))
}

impl MerkleTree {
    /// Build a Merkle tree from leaf hex strings. Leaves are expected to be hex-encoded
    /// 32-byte values (but any hex is fine).
    pub fn new(mut leaves: Vec<String>) -> Self {
        // If there are zero leaves, define the root as hash of empty bytes
        if leaves.is_empty() {
            let empty = Sha256::digest(&[]);
            return MerkleTree { levels: vec![vec![hex::encode(empty)]] };
        }

        // If odd number, duplicate last leaf to make even
        if leaves.len() % 2 == 1 {
            let last = leaves.last().unwrap().clone();
            leaves.push(last);
        }

        let mut levels = Vec::<Vec<String>>::new();
        levels.push(leaves);

        // Build upper layers
        while levels.last().unwrap().len() > 1 {
            let prev = levels.last().unwrap();
            let mut next = Vec::<String>::new();
            let mut i = 0;
            while i < prev.len() {
                let left = &prev[i];
                let right = &prev[i + 1];
                let parent = combine_hex(left, right).expect("hash combine failed");
                next.push(parent);
                i += 2;
            }
            // if odd (shouldn't happen due to duplication), duplicate last
            if next.len() % 2 == 1 && next.len() > 1 {
                let last = next.last().unwrap().clone();
                next.push(last);
            }
            levels.push(next);
        }

        MerkleTree { levels }
    }

    /// Return the hex string root
    pub fn root(&self) -> String {
        self.levels.last().unwrap()[0].clone()
    }

    /// Produce a Merkle proof for `leaf_index` (0-based). Returns None if out of bounds.
    pub fn gen_proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        if self.levels.is_empty() || self.levels[0].is_empty() {
            return None;
        }
        if leaf_index >= self.levels[0].len() {
            return None;
        }

        let mut sibling_hashes = Vec::new();
        let mut sibling_is_left = Vec::new();
        let mut index = leaf_index;

        for level in 0 .. (self.levels.len() - 1) {
            let nodes = &self.levels[level];
            // sibling index
            let is_right = index % 2 == 1;
            let sibling_index = if is_right { index - 1 } else { index + 1 };

            // If sibling index is out of bounds (happens if duplication wasn't applied), use the node itself
            let sibling = if sibling_index < nodes.len() {
                nodes[sibling_index].clone()
            } else {
                nodes[index].clone()
            };

            // If sibling was to the left of the current node
            sibling_hashes.push(sibling);
            sibling_is_left.push(!is_right);

            // Move to parent index for next level
            index = index / 2;
        }

        Some(MerkleProof {
            sibling_hashes,
            sibling_is_left,
        })
    }

    /// Verify a proof given the leaf (hex), proof and expected root (hex).
    pub fn verify_proof(leaf_hex: &str, proof: &MerkleProof, expected_root_hex: &str) -> bool {
        // Start with the leaf bytes
        let mut current_hex = leaf_hex.to_string();

        for (sibling_hex, sibling_is_left) in proof.sibling_hashes.iter().zip(proof.sibling_is_left.iter()) {
            let parent_hex = if *sibling_is_left {
                // sibling on left: parent = hash(sibling || current)
                combine_hex(sibling_hex, &current_hex)
            } else {
                // sibling on right: parent = hash(current || sibling)
                combine_hex(&current_hex, sibling_hex)
            };
            match parent_hex {
                Some(p) => current_hex = p,
                None => return false,
            }
        }

        current_hex == expected_root_hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;

    #[test]
    fn build_tree_and_proof_verify() {
        // create deterministic leaf-like hex values
        let leaf1 = hex::encode(Sha256::digest(b"leaf1"));
        let leaf2 = hex::encode(Sha256::digest(b"leaf2"));
        let leaf3 = hex::encode(Sha256::digest(b"leaf3"));
        let leaf4 = hex::encode(Sha256::digest(b"leaf4"));

        let leaves = vec![leaf1.clone(), leaf2.clone(), leaf3.clone(), leaf4.clone()];
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.root();

        // generate proof for leaf index 2 (leaf3)
        let proof = tree.gen_proof(2).expect("proof");
        assert!(MerkleTree::verify_proof(&leaf3, &proof, &root));

        // proof for leaf1
        let proof0 = tree.gen_proof(0).expect("proof0");
        assert!(MerkleTree::verify_proof(&leaf1, &proof0, &root));

        // tampered leaf should fail
        let bad_leaf = hex::encode(Sha256::digest(b"notleaf"));
        assert!(!MerkleTree::verify_proof(&bad_leaf, &proof0, &root));
    }
}
