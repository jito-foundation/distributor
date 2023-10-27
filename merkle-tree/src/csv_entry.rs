use std::{fs::File, path::PathBuf, result};

use serde::{Deserialize, Serialize};

use crate::error::MerkleTreeError;

pub type Result<T> = result::Result<T, MerkleTreeError>;

/// Airdrop Category. Users can belong to multiple categories
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum AirdropCategory {
    Staker,
    Validator,
    Searcher,
}

/// Represents a single entry in a CSV
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CsvEntry {
    /// Pubkey of the claimant; will be responsible for signing the claim
    pub pubkey: String,
    /// amount unlocked, (ui amount)
    pub amount_unlocked: u64,
    /// amount locked, (ui amount)
    pub amount_locked: u64,
    /// Category
    pub category: AirdropCategory,
}

impl CsvEntry {
    pub fn new_from_file(path: &PathBuf) -> Result<Vec<Self>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        let mut entries = Vec::new();
        for result in rdr.deserialize() {
            let record: CsvEntry = result.unwrap();
            entries.push(record);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parsing() {
        let path = PathBuf::from("./test_fixtures/test_csv.csv");
        let entries = CsvEntry::new_from_file(&path).expect("Failed to parse CSV");

        assert_eq!(entries.len(), 3);

        assert_eq!(
            entries[0].pubkey,
            "4SX6nqv5VRLMoNfYM5phvHgcBNcBEwUEES4qPPjf1EqS"
        );
        assert_eq!(entries[0].amount_unlocked, 1000);
        assert_eq!(entries[0].amount_locked, 500);
        assert_eq!(entries[0].category, AirdropCategory::Staker);
    }
}
