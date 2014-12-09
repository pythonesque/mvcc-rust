use {
    CommandId,
    Datum,
    ItemPointerData,
    Oid,
};
use trans::{
    SpecialTransactionId,
    TransactionId,
    TransactionIdResult,
    ValidTransactionId,
};

pub enum Combo {
    Combo,
    Max
}

#[deriving(Show)]
#[repr(C,packed)]
pub struct HeapTupleFields {
    /// inserting xact ID
    t_xmin: TransactionId,
    /// deleting or locking xact ID
    t_xmax: TransactionId,
    /// inserting or deleting command ID, or both
    t_cid: CommandId,
}

#[deriving(Show)]
#[repr(C)]
pub struct NormalTupleHeaderData {
    t_heap: HeapTupleFields,
    /// current t_ctid of this or newer tuple
    t_ctid: ItemPointerData,
}

#[deriving(Show)]
pub struct MinimalTupleHeaderData; // unit struct

bitflags! {
    #[deriving(Show)]
    flags HeapInfoMask: u16 {
        //const HEAP_HASNULL  =           0x0001, // has null attribute(s)
        //const HEAP_HASVARWIDTH =        0x0002, // has variable-width attribute(s)
        //const HEAP_HASEXTERNAL =        0x0004, // has external stored attribute(s)
        //const HEAP_HASOID =             0x0008, // has an object-id field
        /// xmax is a key-shared locker
        const HEAP_XMAX_KEYSHR_LOCK =   0x0010,
        /// t_cid is a combo cid
        const HEAP_COMBOCID =           0x0020,
        /// xmax is exclusive locker
        const HEAP_XMAX_EXCL_LOCK =     0x0040,
        /// xmax, if valid, is only a locker
        const HEAP_XMAX_LOCK_ONLY =     0x0080,

        /// xmax is a shared locker
        const HEAP_XMAX_SHR_LOCK = HEAP_XMAX_EXCL_LOCK.bits | HEAP_XMAX_KEYSHR_LOCK.bits,
        const HEAP_LOCK_MASK = HEAP_XMAX_SHR_LOCK.bits | HEAP_XMAX_EXCL_LOCK.bits |
                               HEAP_XMAX_KEYSHR_LOCK.bits,
        /// t_xmin committed
        const HEAP_XMIN_COMMITTED =   0x0100,
        /// t_xmin invalid/aborted
        const HEAP_XMIN_INVALID =     0x0200,
        const HEAP_XMIN_FROZEN=      HEAP_XMIN_COMMITTED.bits|HEAP_XMIN_INVALID.bits,
        /// t_xmax committed
        const HEAP_XMAX_COMMITTED =   0x0400,
        /// t_xmax invalid/aborted
        const HEAP_XMAX_INVALID =     0x0800,
        /// t_xmax is a MultiXactId
        const HEAP_XMAX_IS_MULTI =    0x1000,
        /// this is UPDATEd version of row
        const HEAP_UPDATED =          0x2000,
        /*const HEAP_MOVED_OFF =        0x4000,  /* moved to another place by pre-9.0
                                               * VACUUM FULL; kept for binary
                                               * upgrade support */
        const HEAP_MOVED_IN =         0x8000,  /* moved from another place by pre-9.0
                                               * VACUUM FULL; kept for binary
                                               * upgrade support */
        const HEAP_MOVED = HEAP_MOVED_OFF.bits | HEAP_MOVED_IN.bits,*/
        /// visibility-related bits
        const HEAP_XACT_MASK = 0xFFF0,

        /// turn these all off when Xmax is to change
        const HEAP_XMAX_BITS = HEAP_XMAX_COMMITTED.bits | HEAP_XMAX_INVALID.bits |
                               HEAP_XMAX_IS_MULTI.bits | HEAP_LOCK_MASK.bits |
                               HEAP_XMAX_LOCK_ONLY.bits
    }
}

impl HeapInfoMask {
    #[inline]
    pub fn xmax_is_locked_only(&self) -> bool {
        !(*self & HEAP_XMAX_LOCK_ONLY).is_empty() ||
        *self & (HEAP_XMAX_IS_MULTI | HEAP_LOCK_MASK) == HEAP_XMAX_EXCL_LOCK
    }

    #[inline]
    pub fn xmax_is_shr_locked(&self) -> bool {
        (*self & HEAP_LOCK_MASK) == HEAP_XMAX_SHR_LOCK
    }

    #[inline]
    pub fn xmax_is_excl_locked(&self) -> bool {
        (*self & HEAP_LOCK_MASK) == HEAP_XMAX_EXCL_LOCK
    }

    #[inline]
    pub fn xmax_is_keyshr_locked(&self) -> bool {
        (*self & HEAP_LOCK_MASK) == HEAP_XMAX_KEYSHR_LOCK
    }
}

bitflags! {
    #[deriving(Show)]
    flags HeapInfoMask2: u16 {
        //const HEAP_NATTS_MASK =     0x07FF, // 11 bits for number of attributes
        // bits 0x1800 are available
        //const HEAP_KEYS_UPDATED =   0x2000, // tuple was updated and key cols modified, or tuple
        //                                    // deleted
        //const HEAP_HOT_UPDATED =    0x4000, // tuple was HOT-updated
        //const HEAP_ONLY_TUPLE =     0x8000, // this is heap-only tuple
        /// visibility-related bits
        const HEAP2_XACT_MASK =     0xE000,

        /*
        * HEAP_TUPLE_HAS_MATCH is a temporary flag used during hash joins.  It is
        * only used in tuples that are in the hash table, and those don't need
        * any visibility information, so we can overlay it on a visibility flag
        * instead of using up a dedicated bit.
        */
        //const HEAP_TUPLE_HAS_MATCH = HEAP_ONLY_TUPLE.bits,
    }
}

#[repr(C)]
pub struct HeapTupleHeaderData<T, Sized? D> {
    data_: T,
    // ^ - 18 bytes (normally) - ^
    /// number of attributes + various flags
    t_infomask2: HeapInfoMask2,
    /// various flag bits, see below
    t_infomask: HeapInfoMask,
    //t_hoff: u8, // sizeof header incl. bitmap, padding
    // ^ - 5 bytes, 23 bytes total (normally) - ^
    //bits_: [u8, .. 0], // bitmap of NULLs -- VARIABLE LENGTH
    /// More bits (if necessary) plus user data (suitably aligned)
    rest_: D,
}

fn multi_xact_id_get_update_xid(_xmax: TransactionId, t_infomask: HeapInfoMask) -> TransactionIdResult {
    // TODO: make this type safe so we can avoid assertions.
    debug_assert!((t_infomask & HEAP_XMAX_LOCK_ONLY).is_empty());
    debug_assert!(!(t_infomask & HEAP_XMAX_IS_MULTI).is_empty());
    // Placeholder
    Err(None)
}

impl<Sized? D> HeapTupleHeaderData<NormalTupleHeaderData, D> {
    #[inline]
    pub fn get_raw_xmin(&self) -> TransactionId {
        self.t_heap.t_xmin
    }

    #[inline]
    pub fn get_xmin(&self) -> TransactionIdResult {
        if self.xmin_frozen() {
            Err(Some(SpecialTransactionId::Frozen))
        } else {
            self.get_raw_xmin().to_normal()
        }
    }

    #[inline]
    pub fn set_xmin(&mut self, xid: ValidTransactionId) {
        self.t_heap.t_xmin.store(xid);
    }

    #[inline]
    pub fn xmin_committed(&self) -> bool {
        (self.t_infomask & HEAP_XMIN_COMMITTED).is_empty()
    }

    #[inline]
    pub fn xmin_invalid(&self) -> bool {
        self.t_infomask & (HEAP_XMIN_COMMITTED|HEAP_XMIN_INVALID) == HEAP_XMIN_INVALID
    }

    #[inline]
    pub fn xmin_frozen(&self) -> bool {
        self.t_infomask & HEAP_XMIN_FROZEN == HEAP_XMIN_FROZEN
    }

    #[inline]
    pub fn set_xmin_committed(&mut self) {
        // TODO: make this type safe so we can avoid assertions.
        debug_assert!(!self.xmin_invalid());
        self.t_infomask = self.t_infomask | HEAP_XMIN_COMMITTED;
    }

    #[inline]
    pub fn set_xmin_invalid(&mut self) {
        // TODO: make this type safe so we can avoid assertions.
        debug_assert!(!self.xmin_committed());
        self.t_infomask = self.t_infomask | HEAP_XMIN_INVALID;
    }

    #[inline]
    pub fn set_xmin_frozen(&mut self) {
        // TODO: make this type safe so we can avoid assertions.
        debug_assert!(!self.xmin_invalid());
        self.t_infomask = self.t_infomask | HEAP_XMIN_FROZEN;
    }

    // This is probably unsafe without checking hint bits...
    // FIXME: make this type safe
    pub fn get_raw_update_xid(&self) -> TransactionIdResult {
        multi_xact_id_get_update_xid(self.get_raw_xmax(), self.t_infomask)
    }

    #[inline]
    pub fn get_update_xid(&self) -> TransactionIdResult {
        if (self.t_infomask & HEAP_XMAX_INVALID).is_empty() &&
           !(self.t_infomask & HEAP_XMAX_IS_MULTI).is_empty() &&
           (self.t_infomask & HEAP_XMAX_LOCK_ONLY).is_empty() {
            self.get_raw_update_xid()
        } else {
            self.get_raw_xmax().to_normal()
        }
    }

    #[inline]
    pub fn get_raw_xmax(&self) -> TransactionId {
        self.t_heap.t_xmax
    }

    #[inline]
    pub fn set_xmax(&mut self, xid: ValidTransactionId) {
        self.data_.t_heap.t_xmax.store(xid);
    }

    #[inline]
    pub fn get_raw_command_id(&self) -> CommandId {
        self.t_heap.t_cid
    }

    #[inline]
    pub fn set_cmin(&mut self, cid: CommandId) {
        // // This will probably be harder to make typesafe. TODO: consider it.
        //debug_assert!((self.t_infomask & HEAP_MOVED).is_empty());
        self.t_heap.t_cid = cid;
        self.t_infomask = self.t_infomask & !HEAP_COMBOCID;
    }

    #[inline]
    pub fn set_cmax(&mut self, cid: CommandId, iscombo: Combo) {
        // // This will probably be harder to make typesafe. TODO: consider it.
        // debug_assert!((self.t_infomask & HEAP_MOVED).is_empty());
        self.t_heap.t_cid = cid;
        match iscombo {
            Combo::Combo => self.t_infomask = self.t_infomask | HEAP_COMBOCID,
            Combo::Max => self.t_infomask = self.t_infomask & !HEAP_COMBOCID,
        }
    }
}

impl<T, Sized? D> Deref<T> for HeapTupleHeaderData<T, D> {
    #[inline]
    fn deref(&self) -> &T { &self.data_ }
}

impl<T, Sized? D> DerefMut<T> for HeapTupleHeaderData<T, D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T { &mut self.data_ }
}

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

pub type HeapTupleIndirectData<'a, T, U = NormalTupleHeaderData, D = [Datum]> =
    HeapTupleHeader<T, &'a HeapTupleHeaderData<U, D>>;

pub type HeapTupleContiguousData<T, U = NormalTupleHeaderData, D = [Datum]> =
    HeapTupleHeader<T, HeapTupleHeaderData<U, D>>;

pub const MAXIMUM_ALIGNOF: uint = 8;

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
    use Datum;
    use super::{
        HeapTupleContiguousData,
        HeapTupleIndirectData,
        HeapTupleDisk,
        HeapTupleHeaderData,
        HeapTupleTemp,
        HeapInfoMask2,
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
            t_infomask2: HeapInfoMask2::empty(),
            t_infomask: super::HEAP_LOCK_MASK,
            //t_hoff: 0,
            //bits_: [],
            rest_: [1],
        };
        println!("{}: {}", &tuple.rest_, tuple.t_infomask);
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
