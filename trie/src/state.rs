#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: String,
    pub method: String,
    pub program_id: String,
    pub data_key: String,
    pub data: String,
    pub public_key: String,
    pub alias: String,
    pub timestamp: u64,
    pub version: String,
}

#[derive(Debug, Default, Clone)]
pub struct TransactionReceipt {
    pub hash: String,
    pub program_id: String,
    pub status: u64,
    pub timestamp: u64,
    pub error_text: String,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub data_key: String,
    pub program_id: String,
    pub alias: String,
    pub version: String,
    pub cid: String,
    pub public_key: String,
    pub loose: u64,
}

#[derive(Debug, Clone)]
pub struct MetaContract {
    pub program_id: String,
    pub public_key: String,
    pub cid: String,
}
