use rlp::{Decodable, Encodable, Rlp, RlpStream};

use crate::state::Transaction;

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(8)
            .append(&self.hash)
            .append(&self.method)
            .append(&self.program_id)
            .append(&self.data_key)
            .append(&self.data)
            .append(&self.public_key)
            .append(&self.alias)
            .append(&self.timestamp)
            .append(&self.version);
    }
}

impl Decodable for Transaction {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Transaction {
            hash: rlp.val_at(0)?,
            method: rlp.val_at(1)?,
            program_id: rlp.val_at(2)?,
            data_key: rlp.val_at(3)?,
            data: rlp.val_at(4)?,
            public_key: rlp.val_at(5)?,
            alias: rlp.val_at(6)?,
            timestamp: rlp.val_at(7)?,
            version: rlp.val_at(8)?,
        })
    }
}
