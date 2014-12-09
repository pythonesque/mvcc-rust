#![feature(asm)]
#![feature(default_type_params)]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(thread_local)]
#![feature(unsafe_destructor)]

#[cfg(test)] extern crate test;

macro_rules! with_offset(($ty:ty,$field:ident,$data:ident,$b:expr) => {
unsafe {
    let $data = 0 as *const $ty;
    let $field = &(*$data).$field;
    let result = $b;
    result
}
})

macro_rules! offset_of(($ty:ty,$field:ident) => {
with_offset!($ty, $field, data, ($field as *const _ as uint) - (data as uint))
})

macro_rules! min_align_of_offset(($ty:ty,$field:ident) => {
with_offset!($ty, $field, data, ::std::mem::min_align_of_val($field))
})

/*thread_local! {
    pub static TIMER: Timer = Timer::new().unwrap()
}*/

pub mod multixact;
pub mod heap;
mod s_lock;
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
