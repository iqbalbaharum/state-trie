use rlp::{Decodable, Encodable, Rlp, RlpStream};

use crate::state::Metadata;

impl Encodable for Metadata {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(7)
            .append(&self.data_key)
            .append(&self.program_id)
            .append(&self.alias)
            .append(&self.version)
            .append(&self.cid)
            .append(&self.public_key)
            .append(&self.loose);
    }
}

impl Decodable for Metadata {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Metadata {
            data_key: rlp.val_at(0)?,
            program_id: rlp.val_at(1)?,
            alias: rlp.val_at(2)?,
            version: rlp.val_at(3)?,
            cid: rlp.val_at(4)?,
            public_key: rlp.val_at(5)?,
            loose: rlp.val_at(6)?,
        })
    }
}
