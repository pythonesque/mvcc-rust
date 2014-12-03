#![feature(default_type_params)]
#![feature(macro_rules)]

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

#[deriving(Show)]
#[repr(packed)]
pub struct HeapTupleFields {
    t_xmin: TransactionId, // inserting xact ID
    t_xmax: TransactionId, // deleting or locking xact ID
    t_cid: CommandId, // inserting or deleting command ID, or both
}

#[deriving(Show)]
pub struct NormalTupleHeaderData {
    t_heap: HeapTupleFields,
    t_ctid: ItemPointerData, // current t_ctid of this or newer tuple
}

#[deriving(Show)]
pub struct MinimalTupleHeaderData; // unit struct

#[repr(C)]
pub struct HeapTupleHeaderData<T, Sized? D> {
    data_: T,
    // ^ - 18 bytes (normally) - ^
    t_infomask2: u16, // number of attributes + various flags
    t_infomask: u16, // various flag bits, see below
    t_hoff: u8, // sizeof header incl. bitmap, padding
    // ^ - 5 bytes, 23 bytes total (normally) - ^
    bits_: [u8, .. 0], // bitmap of NULLs -- VARIABLE LENGTH
    rest_: D, // More bits (if necessary) plus user data (suitably aligned)
}

impl<T, Sized? D> Deref<T> for HeapTupleHeaderData<T, D> {
    #[inline]
    fn deref(&self) -> &T { &self.data_ }
}

/*impl<T> DerefMut<T> for HeapTupleHeaderData<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T { &mut self.t_data_ }
}*/

#[repr(C)]
pub struct HeapTupleHeader<T, Sized? U> {
    t_len: u32, // length of *t_data
    header_: T,
    t_data: U, // -> tuple header and data
}

pub struct HeapTupleTemp;

pub struct HeapTupleDisk {
    t_self: ItemPointerData, // SelfItemPointer
    t_table_oid: Oid, // table the tuple came from
}

impl<T, U> Deref<T> for HeapTupleHeader<T, U> {
    fn deref(&self) -> &T {
        &self.header_
    }
}

impl<T, U> DerefMut<T> for HeapTupleHeader<T, U> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.header_
    }
}

pub type HeapTupleIndirectData<'a, T, U = NormalTupleHeaderData, D = [Datum]> = HeapTupleHeader<T, &'a HeapTupleHeaderData<U, D>>;

pub type HeapTupleContiguousData<T, U = NormalTupleHeaderData, D = [Datum]> = HeapTupleHeader<T, HeapTupleHeaderData<U, D>>;

pub const MAXIMUM_ALIGNOF: uint = 8;

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

// (offsetof(HeapTupleHeaderData, t_infomask2) - sizeof(uint32)) % MAXIMUM_ALIGNOF
pub const MINIMAL_TUPLE_PADDING: uint = 6;

/*pub struct Version {
    version: uint,
}

pub struct Transaction {
    version: Version,
}

pub const BUFFER_SIZE: uint = 512;

struct Buffer {
    buffer: Arc<RwLock<[u8, .. BUFFER_SIZE]>>,
}

struct Page {
    : UnsafeCell<T>,
}

struct Database {
    pages: Pages,
}

pub struct TransactionManager {
    transactions: *T,
    database: Database,
}*/

#[cfg(test)]
mod tests {
    use std::mem;
    use std::u32;
    use super::{
        Datum,
        HeapTupleContiguousData,
        HeapTupleIndirectData,
        HeapTupleDisk,
        HeapTupleHeaderData,
        HeapTupleTemp,
        MinimalTupleHeaderData,
        NormalTupleHeaderData,
    };

    #[test]
    fn maximum_alignof() {
        assert_eq!(mem::align_of::<uint>(), super::MAXIMUM_ALIGNOF);
    }

    #[test]
    fn heap_form_tuple() {
        let tuple: &HeapTupleHeaderData<MinimalTupleHeaderData, [Datum]> = &HeapTupleHeaderData {
            data_: MinimalTupleHeaderData,
            // ^ - 18 bytes (normally) - ^
            t_infomask2: 0,
            t_infomask: 0,
            t_hoff: 0,
            bits_: [],
            rest_: [1],
        };
        println!("{}", &tuple.rest_);
        //assert_eq!(mem::align_of_val(&tuple.rest_), super::MAXIMUM_ALIGNOF);
    }

    #[test]
    // (offsetof(HeapTupleHeaderData, t_infomask2) - sizeof(uint32)) % MAXIMUM_ALIGNOF
    fn minimal_tuple_padding() {
        type C<T, U, D> = HeapTupleContiguousData<T, U, D>;
        type I<'a, T, U, D> = HeapTupleIndirectData<'a, T, U, D>;
        type T = HeapTupleTemp;
        type S = HeapTupleDisk;
        type M = MinimalTupleHeaderData;
        type N = NormalTupleHeaderData;
        type D = [Datum, .. 0];
        let offset = offset_of!(HeapTupleHeaderData<M, D>, rest_);
        let align = min_align_of_offset!(HeapTupleHeaderData<M, D>, rest_);
        println!("{} {} {}", offset, mem::size_of::<HeapTupleHeaderData<M, D>>(), align);
        assert_eq!(align, super::MAXIMUM_ALIGNOF);
        let offset = offset_of!(HeapTupleHeaderData<N, D>, rest_);
        let align = min_align_of_offset!(HeapTupleHeaderData<N, D>, rest_);
        println!("{} {} {}", offset, mem::size_of::<HeapTupleHeaderData<N, D>>(), align);
        assert_eq!(align, super::MAXIMUM_ALIGNOF);

        let offset = offset_of!(C<T, M, D>, t_data);
        println!("{} {}", offset, mem::size_of::<C<T, M, D>>());
        let offset = offset_of!(I<T, N, D>, t_data);
        println!("{} {}", offset, mem::size_of::<I<T, N, D>>());
        let offset = offset_of!(I<S, N, D>, t_data);
        println!("{} {}", offset, mem::size_of::<I<S, N, D>>());
        let offset = offset_of!(C<S, M, D>, t_data);
        println!("{} {}", offset, mem::size_of::<C<S, M, D>>());
        let offset = offset_of!(C<T, N, D>, t_data);
        println!("{} {}", offset, mem::size_of::<C<T, N, D>>());
        let offset = offset_of!(C<S, N, D>, t_data);
        println!("{} {}", offset, mem::size_of::<C<S, N, D>>());

        let offset = offset_of!(HeapTupleHeaderData<N, D>, t_infomask2);
        //println!("{} {}", offset, mem::size_of::<HeapTupleHeaderData<N, D>>());
        let minimal_tuple_padding = (offset - u32::BYTES) % super::MAXIMUM_ALIGNOF;
        assert_eq!(minimal_tuple_padding, super::MINIMAL_TUPLE_PADDING);
    }
}
