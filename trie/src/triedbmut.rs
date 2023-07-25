use std::{borrow::Borrow, marker::PhantomData, ops::Range};

use trie_db::{
    nibble_ops,
    node::{NibbleSlicePlan, NodeHandlePlan, NodePlan, Value, ValuePlan},
    triedbmut::ChildReference,
    DBValue, HashDB, HashDBRef, Hasher, NodeCodec, Trie, TrieCache, TrieDB, TrieDBMutBuilder,
    TrieHash, TrieLayout, TrieMut, TrieRecorder,
};

use parity_scale_codec::{Compact, Encode, Error as CodecError, Input, Output};

const EMPTY_TRIE: u8 = 0;
const LEAF_NODE_OFFSET: u8 = 1;
const EXTENSION_NODE_OFFSET: u8 = 128;
const BRANCH_NODE_NO_VALUE: u8 = 254;
const BRANCH_NODE_WITH_VALUE: u8 = 255;
const LEAF_NODE_OVER: u8 = EXTENSION_NODE_OFFSET - LEAF_NODE_OFFSET;
const EXTENSION_NODE_OVER: u8 = BRANCH_NODE_NO_VALUE - EXTENSION_NODE_OFFSET;
const LEAF_NODE_LAST: u8 = EXTENSION_NODE_OFFSET - 1;
const EXTENSION_NODE_LAST: u8 = BRANCH_NODE_NO_VALUE - 1;

/// Children bitmap codec for radix 16 trie.
pub struct Bitmap(u16);
const BITMAP_LENGTH: usize = 2;

/// Reference hasher is a keccak hasher.
pub type RefHasher = keccak_hasher::KeccakHash;

/// Abstraction utility to read from a given bytes slice.
struct ByteSliceInput<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ByteSliceInput<'a> {
    fn new(data: &'a [u8]) -> Self {
        ByteSliceInput { data, offset: 0 }
    }

    fn take(&mut self, count: usize) -> Result<Range<usize>, CodecError> {
        if self.offset + count > self.data.len() {
            return Err("out of data".into());
        }

        let range = self.offset..(self.offset + count);
        self.offset += count;
        Ok(range)
    }
}

impl<'a> Input for ByteSliceInput<'a> {
    fn remaining_len(&mut self) -> Result<Option<usize>, CodecError> {
        let remaining = if self.offset <= self.data.len() {
            Some(self.data.len() - self.offset)
        } else {
            None
        };
        Ok(remaining)
    }

    fn read(&mut self, into: &mut [u8]) -> Result<(), CodecError> {
        let range = self.take(into.len())?;
        into.copy_from_slice(&self.data[range]);
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, CodecError> {
        if self.offset + 1 > self.data.len() {
            return Err("out of data".into());
        }

        let byte = self.data[self.offset];
        self.offset += 1;
        Ok(byte)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum NodeHeader {
    Null,
    Branch(bool),
    Extension(usize),
    Leaf(usize),
}

impl Encode for NodeHeader {
    fn encode_to<T: Output + ?Sized>(&self, output: &mut T) {
        match self {
            NodeHeader::Null => output.push_byte(EMPTY_TRIE),
            NodeHeader::Branch(true) => output.push_byte(BRANCH_NODE_WITH_VALUE),
            NodeHeader::Branch(false) => output.push_byte(BRANCH_NODE_NO_VALUE),
            NodeHeader::Leaf(nibble_count) => {
                output.push_byte(LEAF_NODE_OFFSET + *nibble_count as u8)
            }
            NodeHeader::Extension(nibble_count) => {
                output.push_byte(EXTENSION_NODE_OFFSET + *nibble_count as u8)
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct ExtensionLayout;

impl TrieLayout for ExtensionLayout {
    const USE_EXTENSION: bool = true;
    const ALLOW_EMPTY: bool = false;
    const MAX_INLINE_VALUE: Option<u32> = None;
    type Hash = RefHasher;
    type Codec = ReferenceNodeCodec<RefHasher>;
}

/// NOTE: Following implementation for TrieLayout
/// To complex to code it, so copy and paste from the code below
/// https://github.com/paritytech/trie/blob/master/test-support/reference-trie/src/lib.rs
#[derive(Default, Clone)]
pub struct ReferenceNodeCodec<H>(PhantomData<H>);

impl<H: Hasher> NodeCodec for ReferenceNodeCodec<H> {
    type Error = CodecError;
    type HashOut = H::Out;

    fn hashed_null_node() -> <H as Hasher>::Out {
        H::hash(<Self as NodeCodec>::empty_node())
    }

    fn decode_plan(data: &[u8]) -> ::std::result::Result<NodePlan, Self::Error> {
        let mut input = ByteSliceInput::new(data);
        match NodeHeader::decode(&mut input)? {
            NodeHeader::Null => Ok(NodePlan::Empty),
            NodeHeader::Branch(has_value) => {
                let bitmap_range = input.take(BITMAP_LENGTH)?;
                let bitmap = Bitmap::decode(&data[bitmap_range])?;

                let value = if has_value {
                    let count = <Compact<u32>>::decode(&mut input)?.0 as usize;
                    Some(ValuePlan::Inline(input.take(count)?))
                } else {
                    None
                };
                let mut children = [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None,
                ];
                for i in 0..nibble_ops::NIBBLE_LENGTH {
                    if bitmap.value_at(i) {
                        let count = <Compact<u32>>::decode(&mut input)?.0 as usize;
                        let range = input.take(count)?;
                        children[i] = Some(if count == H::LENGTH {
                            NodeHandlePlan::Hash(range)
                        } else {
                            NodeHandlePlan::Inline(range)
                        });
                    }
                }
                Ok(NodePlan::Branch { value, children })
            }
            NodeHeader::Extension(nibble_count) => {
                let partial = input.take(
                    (nibble_count + (nibble_ops::NIBBLE_PER_BYTE - 1))
                        / nibble_ops::NIBBLE_PER_BYTE,
                )?;
                let partial_padding = nibble_ops::number_padding(nibble_count);
                let count = <Compact<u32>>::decode(&mut input)?.0 as usize;
                let range = input.take(count)?;
                let child = if count == H::LENGTH {
                    NodeHandlePlan::Hash(range)
                } else {
                    NodeHandlePlan::Inline(range)
                };
                Ok(NodePlan::Extension {
                    partial: NibbleSlicePlan::new(partial, partial_padding),
                    child,
                })
            }
            NodeHeader::Leaf(nibble_count) => {
                let partial = input.take(
                    (nibble_count + (nibble_ops::NIBBLE_PER_BYTE - 1))
                        / nibble_ops::NIBBLE_PER_BYTE,
                )?;
                let partial_padding = nibble_ops::number_padding(nibble_count);
                let count = <Compact<u32>>::decode(&mut input)?.0 as usize;
                let value = input.take(count)?;
                Ok(NodePlan::Leaf {
                    partial: NibbleSlicePlan::new(partial, partial_padding),
                    value: ValuePlan::Inline(value),
                })
            }
        }
    }

    fn is_empty_node(data: &[u8]) -> bool {
        data == <Self as NodeCodec>::empty_node()
    }

    fn empty_node() -> &'static [u8] {
        &[EMPTY_TRIE]
    }

    fn leaf_node(partial: impl Iterator<Item = u8>, number_nibble: usize, value: Value) -> Vec<u8> {
        let mut output =
            partial_from_iterator_to_key(partial, number_nibble, LEAF_NODE_OFFSET, LEAF_NODE_OVER);
        match value {
            Value::Inline(value) => {
                Compact(value.len() as u32).encode_to(&mut output);
                output.extend_from_slice(value);
            }
            _ => unimplemented!("unsupported"),
        }
        output
    }

    fn extension_node(
        partial: impl Iterator<Item = u8>,
        number_nibble: usize,
        child: ChildReference<Self::HashOut>,
    ) -> Vec<u8> {
        let mut output = partial_from_iterator_to_key(
            partial,
            number_nibble,
            EXTENSION_NODE_OFFSET,
            EXTENSION_NODE_OVER,
        );
        match child {
            ChildReference::Hash(h) => h.as_ref().encode_to(&mut output),
            ChildReference::Inline(inline_data, len) => {
                (&AsRef::<[u8]>::as_ref(&inline_data)[..len]).encode_to(&mut output)
            }
        };
        output
    }

    fn branch_node(
        children: impl Iterator<Item = impl Borrow<Option<ChildReference<Self::HashOut>>>>,
        maybe_value: Option<Value>,
    ) -> Vec<u8> {
        let mut output = vec![0; BITMAP_LENGTH + 1];
        let mut prefix: [u8; 3] = [0; 3];
        let have_value = match maybe_value {
            Some(Value::Inline(value)) => {
                Compact(value.len() as u32).encode_to(&mut output);
                output.extend_from_slice(value);
                true
            }
            None => false,
            _ => unimplemented!("unsupported"),
        };
        let has_children = children.map(|maybe_child| match maybe_child.borrow() {
            Some(ChildReference::Hash(h)) => {
                h.as_ref().encode_to(&mut output);
                true
            }
            &Some(ChildReference::Inline(inline_data, len)) => {
                inline_data.as_ref()[..len].encode_to(&mut output);
                true
            }
            None => false,
        });
        branch_node_buffered(have_value, has_children, prefix.as_mut());
        output[0..BITMAP_LENGTH + 1].copy_from_slice(prefix.as_ref());
        output
    }

    fn branch_node_nibbled(
        _partial: impl Iterator<Item = u8>,
        _number_nibble: usize,
        _children: impl Iterator<Item = impl Borrow<Option<ChildReference<Self::HashOut>>>>,
        _maybe_value: Option<Value>,
    ) -> Vec<u8> {
        unreachable!("codec with extension branch")
    }
}

fn partial_from_iterator_to_key<I: Iterator<Item = u8>>(
    partial: I,
    nibble_count: usize,
    offset: u8,
    over: u8,
) -> Vec<u8> {
    assert!(nibble_count < over as usize);
    let mut output = Vec::with_capacity(1 + (nibble_count / nibble_ops::NIBBLE_PER_BYTE));
    output.push(offset + nibble_count as u8);
    output.extend(partial);
    output
}

/// Encoding of branch header and children bitmap for any radix.
/// For codec/stream variant with extension.
fn branch_node_buffered<I: Iterator<Item = bool>>(
    has_value: bool,
    has_children: I,
    output: &mut [u8],
) {
    let first = if has_value {
        BRANCH_NODE_WITH_VALUE
    } else {
        BRANCH_NODE_NO_VALUE
    };
    output[0] = first;
    Bitmap::encode(has_children, &mut output[1..]);
}
