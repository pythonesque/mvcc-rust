#![feature(default_type_params)]
#![feature(macro_rules)]

/*use std::mem;
use std::ptr::RawPtr;
use std::raw::Repr;
use std::raw::Slice as RawSlice;
use std::uint;*/

#[deriving(Show)]
//#[repr(C)]
pub struct Oid(u32);

#[deriving(Show)]
//#[repr(C)]
pub struct TransactionId(u32);

#[deriving(Show)]
//#[repr(C)]
pub struct CommandId(u32);

#[deriving(Show)]
//#[repr(C)]
pub struct BlockNumber(u32);

#[deriving(Show)]
//#[repr(C)]
pub struct BlockIdData {
    bi_hi: u16,
    bi_lo: u16,
}

#[deriving(Show)]
//#[repr(C)]
pub struct OffsetNumber(u16);

#[deriving(Show)]
//#[repr(C,packed)]
pub struct ItemPointerData {
    ip_blkid: BlockIdData,
    ip_posid: OffsetNumber,
}

#[deriving(Show)]
//#[repr(C,packed)]
#[repr(packed)]
pub struct HeapTupleFields {
    t_xmin: TransactionId, // inserting xact ID
    t_xmax: TransactionId, // deleting or locking xact ID
    t_cid: CommandId, // inserting or deleting command ID, or both
}

#[deriving(Show)]
//#[repr(C,packed)]
pub struct NormalTupleHeaderData {
    t_heap: HeapTupleFields,
    t_ctid: ItemPointerData, // current t_ctid of this or newer tuple
}

#[deriving(Show)]
//#[repr(C,packed)]
pub struct MinimalTupleHeaderData; // unit struct

#[deriving(Show)]
#[repr(C)]
pub struct HeapTupleHeaderData<T> {
    t_data_: T,
    // ^ - 18 bytes (normally) - ^
    t_infomask2: u16, // number of attributes + various flags
    t_infomask: u16, // various flag bits, see below
    t_hoff: u8, // sizeof header incl. bitmap, padding
    // ^ - 5 bytes, 23 bytes total (normally) - ^
    t_bits_: [u8, .. 1] // bitmap of NULLs -- VARIABLE LENGTH
    // MORE DATA FOLLOWS AT END OF STRUCT
}

impl<T> Deref<T> for HeapTupleHeaderData<T> {
    #[inline]
    fn deref(&self) -> &T { &self.t_data_ }
}

impl<T> DerefMut<T> for HeapTupleHeaderData<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T { &mut self.t_data_ }
}

/*#[deriving(Show)]
pub struct BitMap {
    t_bits: &[u8],
}*/

/*impl<T> HeapTupleHeaderData<T> {
    #[inline]
    fn t_bits_len(&self) -> uint {
        self.len
    }

    pub fn t_bits(&self) -> BitMap {
        let len =
        unsafe { ::std::mem::transmute(self.t_bits_.repr().data
    }
}*/

pub struct HeapTupleHeader<T, U> {
    t_len: u32,
    t_header_: T,
    t_data: U,
}

pub struct HeapTupleTemp;

pub struct HeapTupleDisk {
    t_self: ItemPointerData, // SelfItemPointer
    t_table_oid: Oid, // table the tuple came from
}

impl<T, U> Deref<T> for HeapTupleHeader<T, U> {
    fn deref(&self) -> &T {
        &self.t_header_
    }
}

impl<T, U> DerefMut<T> for HeapTupleHeader<T, U> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.t_header_
    }
}

pub type HeapTupleDirectData<'a, T, U = NormalTupleHeaderData> = HeapTupleHeader<T, &'a HeapTupleHeaderData<U>>;

pub type HeapTupleContiguousData<T, U = NormalTupleHeaderData> = HeapTupleHeader<T, HeapTupleHeaderData<U>>;

/*#[repr(C,packed)]
pub struct HeapTupleHeaderInfo<Sized? T> {
    t_infomask2: u16, // number of attributes + various flags
    t_infomask: u16, // various flag bits, see below
    t_hoff: u8, // sizeof header incl. bitmap, padding
    // ^ - 5 bytes - ^
    t_bits: T // bitmap of NULLs -- VARIABLE LENGTH
}

#[repr(C)]
pub struct HeapTupleHeaderData<Sized? T> {
    t_heap: HeapTupleFields,
    t_ctid: ItemPointerData, // current t_ctid of this or newer tuple
    // ^ - 18 bytes - ^
    // Fields below here must match MinimalTupleData!
    t_info: HeapTupleHeaderInfo<T>,
}

impl Deref<HeapTupleHeaderInfo<[u8]>> for HeapTupleHeaderData<[u8]> {
    fn deref(&self) -> &HeapTupleHeaderInfo<[u8]> {
        &self.t_info
    }
}

/*impl DerefMut<HeapTupleHeaderInfo> for HeapTupleHeaderData {
    pub fn deref_mut(&mut self) -> &mut HeapTupleHeaderInfo {
        &mut self.t_info
    }
}*/

/*impl Repr<HeapTupleHeaderData> for [T] {
}*/*/

pub const MAXIMUM_ALIGNOF: uint = 8;

macro_rules! offset_of(($ty:ty,$field:ident) => {
    unsafe {
        //use std::raw::Repr;
        //use std::ptr::RawPtr;
        //let data = ::std::mem::transmute::<[u8], $ty>(::std::mem::uninitialized());
        //let data = ::std::mem::uninitialized::<$ty>();
        let data = ::std::mem::uninitialized::<$ty>();
        //let data = 0 as *const $ty;

        //let data = ::std::mem::zeroed::<*const $ty>();
        //let offset = (&(*data).$field as *const _).to_uint();
        //let offset = (&(*data).$field as *const _) as uint;//.to_uint();
        //mem::forget(data);
        //let offset = (&(*data).$field as *const _ as uint) - (&data as *const _ as uint);//.to_uint();
        let offset = (&data.$field as *const _ as uint) - (&data as *const _ as uint);//.to_uint();
        mem::forget(data);
        offset
    }
})

// (offsetof(HeapTupleHeaderData, t_infomask2) - sizeof(uint32)) % MAXIMUM_ALIGNOF
pub const MINIMAL_TUPLE_PADDING: uint = 6;

/*#[repr(C)]
pub struct MinimalTupleData<Sized? T> {
    t_len: u32, // actual length of minimal tuple
    mt_padding: [u8, .. MINIMAL_TUPLE_PADDING],
    // ^ - 18 bytes - ^
    // Fields below here must match HeapTupleHeaderData!
    t_info: HeapTupleHeaderInfo<T>,
}

impl Deref<HeapTupleHeaderInfo<[u8]>> for MinimalTupleData<[u8]> {
    fn deref(&self) -> &HeapTupleHeaderInfo<[u8]> {
        &self.t_info
    }
}

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
}*/*/

#[cfg(test)]
mod tests {
    use std::mem;
    //use std::raw::Repr;
    use std::u32;
    use super::{
        HeapTupleContiguousData,
        HeapTupleDirectData,
        HeapTupleDisk,
        //HeapTupleHeader,
        HeapTupleHeaderData,
        //HeapTupleHeaderInfo,
        HeapTupleTemp,
        //MinimalTupleData,
        MinimalTupleHeaderData,
        NormalTupleHeaderData,
    };

    #[test]
    fn maximum_alignof() {
        assert_eq!(mem::align_of::<uint>(), super::MAXIMUM_ALIGNOF);
    }

    #[test]
    // (offsetof(HeapTupleHeaderData, t_infomask2) - sizeof(uint32)) % MAXIMUM_ALIGNOF
    fn minimal_tuple_padding() {
        /*let offset = unsafe {
            let data = HeapTupleHeaderData {
                t_heap: mem::uninitialized(),
                t_ctid: mem::uninitialized(),
                t_info: HeapTupleHeaderInfo {
                    t_infomask2: mem::uninitialized(),
                    t_infomask: mem::uninitialized(),
                    t_hoff: mem::uninitialized(),
                    t_bits: [0u8, .. 0],
                }
            };
            let HeapTupleHeaderData {
                t_heap: t_heap,
                t_ctid: t_ctid,
                t_info: /*HeapTupleHeaderInfo {
                    t_infomask2: t_infomask2,
                    t_infomask: t_infomask,
                    t_hoff: t_hoff,
                    t_bits: ref t_bits,
                }*/ref t_info
            } = data;
            /*let HeapTupleHeaderInfo {
                t_infomask2: t_infomask2,
                t_infomask: t_infomask,
                t_hoff: t_hoff,
                t_bits: ref t_bits,
            } = t_info;*/
            //let t_bits: &[u8] = &*t_bits;
            /*let t_info: &HeapTupleHeaderInfo<[u8]> = &HeapTupleHeaderInfo {
                t_infomask2: t_infomask2,
                t_infomask: t_infomask,
                t_hoff: t_hoff,
                t_bits: *t_bits,
            };*/

            let data: &HeapTupleHeaderData<[u8]> = &HeapTupleHeaderData {
                t_heap: t_heap,
                t_ctid: t_ctid,
                t_info: *t_info,
            };
            println!("{}", data.t_bits.len());
            //let data = mem::transmute::<_, &HeapTupleHeaderData<[u8]>>(&data);
            //let offset = (&(*data).$field as *const _).to_uint();
            //let offset = (&data.t_info as *const _ as uint) - (data as *const _ as uint);
            //let offset = (&data.t_info as *const _ as uint) - (&data as *const _ as uint);
            //mem::forget(data);
            //offset
            0
        };*/

        //let offset = offset_of!(HeapTupleHeaderData<MinimalTupleHeaderData>, t_infomask2);
        //println!("{} {}", offset, mem::size_of::<HeapTupleHeaderData<MinimalTupleHeaderData>>());
        let offset = offset_of!(HeapTupleContiguousData<HeapTupleTemp, MinimalTupleHeaderData>, t_data);
        println!("{} {}", offset, mem::size_of::<HeapTupleContiguousData<HeapTupleTemp, MinimalTupleHeaderData>>());
        let offset = offset_of!(HeapTupleDirectData<HeapTupleTemp>, t_data);
        println!("{} {}", offset, mem::size_of::<HeapTupleDirectData<HeapTupleTemp>>());
        let offset = offset_of!(HeapTupleDirectData<HeapTupleDisk>, t_data);
        println!("{} {}", offset, mem::size_of::<HeapTupleDirectData<HeapTupleDisk>>());
        let offset = offset_of!(HeapTupleContiguousData<HeapTupleDisk, MinimalTupleHeaderData>, t_data);
        println!("{} {}", offset, mem::size_of::<HeapTupleContiguousData<HeapTupleDisk, MinimalTupleHeaderData>>());
        let offset = offset_of!(HeapTupleContiguousData<HeapTupleTemp>, t_data);
        println!("{} {}", offset, mem::size_of::<HeapTupleContiguousData<HeapTupleTemp>>());
        let offset = offset_of!(HeapTupleContiguousData<HeapTupleDisk>, t_data);
        println!("{} {}", offset, mem::size_of::<HeapTupleContiguousData<HeapTupleDisk>>());

        let offset = offset_of!(HeapTupleHeaderData<NormalTupleHeaderData>, t_infomask2);
        //println!("{} {}", offset, mem::size_of::<HeapTupleHeaderData<NormalTupleHeaderData>>());
        let minimal_tuple_padding = (offset - u32::BYTES) % super::MAXIMUM_ALIGNOF;
        assert_eq!(minimal_tuple_padding, super::MINIMAL_TUPLE_PADDING);

        /*let offset = offset_of!(HeapTupleHeaderData<[u8, ..0]>, t_info);
        let minimal_tuple_padding = (offset - u32::BYTES) % super::MAXIMUM_ALIGNOF;
        assert_eq!(minimal_tuple_padding, super::MINIMAL_TUPLE_PADDING);
        let offset = offset_of!(MinimalTupleData<[u8, ..0]>, t_info);
        assert_eq!(minimal_tuple_padding, super::MINIMAL_TUPLE_PADDING);*/
    }
}
