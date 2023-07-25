use crate::{
    state::{MetaContract, Metadata, Transaction, TransactionReceipt},
    triedbmut::RefHasher,
};
use trie_db::{node::NodePlan, NodeCodec};

struct MetadataCodec;
struct TransactionCodec;
struct TransactionReceiptCodec;
struct MetaContractCodec;

impl MetadataCodec {
    fn encode(metadata: &Metadata) -> Vec<u8> {
        let mut rlp_stream = RlpStream::new();
        metadata.rlp_append(&mut rlp_stream);
        rlp_stream.out()
    }

    fn decode(encoded_data: &[u8]) -> Result<Metadata, rlp::DecoderError> {
        let rlp = rlp::Rlp::new(encoded_data);
        Metadata::decode(&rlp)
    }
}

impl TransactionCodec {
    fn encode(transaction: &Transaction) -> Vec<u8> {
        let mut rlp_stream = RlpStream::new();
        transaction.rlp_append(&mut rlp_stream);
        rlp_stream.out()
    }

    fn decode(encoded_data: &[u8]) -> Result<Transaction, rlp::DecoderError> {
        let rlp = rlp::Rlp::new(encoded_data);
        Transaction::decode(&rlp)
    }
}

impl TransactionReceiptCodec {
    fn encode(transaction_receipt: &TransactionReceipt) -> Vec<u8> {
        let mut rlp_stream = RlpStream::new();
        transaction_receipt.rlp_append(&mut rlp_stream);
        rlp_stream.out()
    }

    fn decode(encoded_data: &[u8]) -> Result<TransactionReceipt, rlp::DecoderError> {
        let rlp = rlp::Rlp::new(encoded_data);
        TransactionReceipt::decode(&rlp)
    }
}

impl MetaContractCodec {
    fn encode(contract: &MetaContract) -> Vec<u8> {
        let mut rlp_stream = RlpStream::new();
        contract.rlp_append(&mut rlp_stream);
        rlp_stream.out()
    }

    fn decode(encoded_data: &[u8]) -> Result<MetaContract, rlp::DecoderError> {
        let rlp = rlp::Rlp::new(encoded_data);
        MetaContract::decode(&rlp)
    }
}
