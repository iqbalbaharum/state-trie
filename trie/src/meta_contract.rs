use rlp::{Decodable, Encodable, Rlp, RlpStream};

use crate::state::MetaContract;

impl Encodable for MetaContract {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3)
            .append(&self.program_id)
            .append(&self.public_key)
            .append(&self.cid);
    }
}

impl Decodable for MetaContract {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(MetaContract {
            program_id: rlp.val_at(0)?,
            public_key: rlp.val_at(1)?,
            cid: rlp.val_at(2)?,
        })
    }
}
