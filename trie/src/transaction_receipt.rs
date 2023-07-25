use rlp::{Decodable, Encodable, Rlp, RlpStream};

use crate::state::TransactionReceipt;

impl Encodable for TransactionReceipt {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(6)
            .append(&self.hash)
            .append(&self.program_id)
            .append(&self.status)
            .append(&self.timestamp)
            .append(&self.error_text)
            .append(&self.data);
    }
}

impl Decodable for TransactionReceipt {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(TransactionReceipt {
            hash: rlp.val_at(0)?,
            program_id: rlp.val_at(1)?,
            status: rlp.val_at(2)?,
            timestamp: rlp.val_at(3)?,
            error_text: rlp.val_at(4)?,
            data: rlp.val_at(5)?,
        })
    }
}
