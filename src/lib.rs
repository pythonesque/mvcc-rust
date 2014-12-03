#![feature(default_type_params)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]

pub mod heap;
pub mod trans;

#[deriving(Show)]
pub struct Oid(u32);

#[deriving(Show)]
pub struct CommandId(u32);

pub type Datum = uint;

#[deriving(Show)]
pub struct BlockNumber(u32);

#[deriving(Show)]
#[repr(C)]
pub struct BlockIdData {
    bi_hi: u16,
    bi_lo: u16,
}

#[deriving(Show)]
#[repr(C)]
pub struct OffsetNumber(u16);

#[deriving(Show)]
#[repr(C)]
pub struct ItemPointerData {
    ip_blkid: BlockIdData,
    ip_posid: OffsetNumber,
}
