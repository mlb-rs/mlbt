use crate::live::{FullPlayer, PrimaryPosition};
use chrono::NaiveDate;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RosterType {
    Active,
    FortyMan,
}

impl fmt::Display for RosterType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RosterType::Active => write!(f, "active"),
            RosterType::FortyMan => write!(f, "40Man"),
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct RosterResponse {
    pub roster: Vec<RosterEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterEntry {
    pub person: FullPlayer,
    pub jersey_number: Option<String>,
    pub position: PrimaryPosition,
    pub status: RosterStatus,
    pub parent_team_id: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct RosterStatus {
    pub code: String,
    pub description: String,
}

#[derive(Default, Debug, Deserialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: u64,
    pub person: Option<TransactionEntity>,
    pub from_team: Option<TransactionEntity>,
    pub to_team: Option<TransactionEntity>,
    #[serde(default, with = "crate::serde_dates::optional_date")]
    pub date: Option<NaiveDate>,
    #[serde(default, with = "crate::serde_dates::optional_date")]
    pub effective_date: Option<NaiveDate>,
    #[serde(default, with = "crate::serde_dates::optional_date")]
    pub resolution_date: Option<NaiveDate>,
    pub type_code: Option<String>,
    pub type_desc: Option<String>,
    pub description: Option<String>,
}

/// Lightweight id+name reference used in transaction records.
/// Separate from `IdNameLink` because person ids exceed u16.
#[derive(Debug, Deserialize)]
pub struct TransactionEntity {
    pub id: u64,
    pub name: Option<String>,
}
