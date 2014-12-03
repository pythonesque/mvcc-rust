#![feature(default_type_params)]
#![feature(macro_rules)]

pub mod heap;

#[deriving(Show)]
pub struct Oid(u32);

#[deriving(Show)]
pub struct TransactionId(u32);

#[deriving(Show)]
pub struct CommandId(u32);

pub type Datum = uint;

#[deriving(Show)]
pub struct BlockNumber(u32);

#[deriving(Show)]
pub struct BlockIdData {
    bi_hi: u16,
    bi_lo: u16,
}

#[deriving(Show)]
pub struct OffsetNumber(u16);

#[deriving(Show)]
pub struct ItemPointerData {
    ip_blkid: BlockIdData,
    ip_posid: OffsetNumber,
}
