#![macro_escape]

use s_lock::SpinLock;

#[deriving(Show)]
pub struct LWLockHeader {
    /// T if ok to release waiters
    release_ok: bool,
    /// # of exclusive holders (0 or 1)
    exclusive: u8,
    /// # of shared holders (0..MaxBackends)
    shared: u32,
    //tranche: u32, /// tranche ID
    /*head: Cell<*const Proc>, /// head of list of waiting PROCs
    tail: Cell<*const Proc>, /// tail of list of waiting PROCs
    // tail is undefined when head is NULL*/
}

pub struct LWLockInner<T> {
    pub header: LWLockHeader,
    pub data: T,
}

#[repr(C)]
pub struct LWLock<T> {
    #[doc(hidden)] pub mutex: SpinLock<(), LWLockInner<T>>, // Protects LWLock and queue of PROCs
}

macro_rules! lwlock_init(
    ($data:expr) => (
        ::lwlock::LWLock {
            mutex: spin_lock_init!((),
                ::lwlock::LWLockInner {
                    header: ::lwlock::LWLockHeader {
                        release_ok: true,
                        exclusive: 0,
                        shared: 0,
                    },
                    data: $data,
                }
            )
        }
    )
)

/*pub const LWLOCK_PADDED_SIZE = 32; // power of 2
pub const LWLOCK_PADDING = 12; // LWLOCK_PADDED_SIZE - mem::size_of::<LWLock>

#[repr(C)]
pub struct LWLockPadded {
    lock: LWLock,
    _pad: [u8, .. LWLOCK_PADDING]
}

impl Deref<LWLock> for LWLockPadded {
    fn deref(&self) -> &LWLock {
        &*self.lock
    }
}

impl DerefMut<LWLock> for LWLockPadded {
    fn deref_mut(&mut self) -> &mut LWLock {
        &mut *self.lock
    }
}

pub const LWLOCK_PADDED_INIT: LWLockPadded = LWLockPadded {
    lock: LWLOCK_INIT,
    _pad: [u8, .. LWLOCK_PADDING]
}*/

/*#define ShmemIndexLock				(&MainLWLockArray[1].lock)
#define OidGenLock					(&MainLWLockArray[2].lock)
#define XidGenLock					(&MainLWLockArray[3].lock)
#define ProcArrayLock				(&MainLWLockArray[4].lock)
#define SInvalReadLock				(&MainLWLockArray[5].lock)
#define SInvalWriteLock				(&MainLWLockArray[6].lock)
#define WALBufMappingLock			(&MainLWLockArray[7].lock)
#define WALWriteLock				(&MainLWLockArray[8].lock)
#define ControlFileLock				(&MainLWLockArray[9].lock)
#define CheckpointLock				(&MainLWLockArray[10].lock)
#define CLogControlLock				(&MainLWLockArray[11].lock)
#define SubtransControlLock			(&MainLWLockArray[12].lock)
#define MultiXactGenLock			(&MainLWLockArray[13].lock)
#define MultiXactOffsetControlLock	(&MainLWLockArray[14].lock)
#define MultiXactMemberControlLock	(&MainLWLockArray[15].lock)
#define RelCacheInitLock			(&MainLWLockArray[16].lock)
#define CheckpointerCommLock		(&MainLWLockArray[17].lock)
#define TwoPhaseStateLock			(&MainLWLockArray[18].lock)
#define TablespaceCreateLock		(&MainLWLockArray[19].lock)
#define BtreeVacuumLock				(&MainLWLockArray[20].lock)
#define AddinShmemInitLock			(&MainLWLockArray[21].lock)
#define AutovacuumLock				(&MainLWLockArray[22].lock)
#define AutovacuumScheduleLock		(&MainLWLockArray[23].lock)
#define SyncScanLock				(&MainLWLockArray[24].lock)
#define RelationMappingLock			(&MainLWLockArray[25].lock)
#define AsyncCtlLock				(&MainLWLockArray[26].lock)
#define AsyncQueueLock				(&MainLWLockArray[27].lock)
#define SerializableXactHashLock	(&MainLWLockArray[28].lock)
#define SerializableFinishedListLock		(&MainLWLockArray[29].lock)
#define SerializablePredicateLockListLock	(&MainLWLockArray[30].lock)
#define OldSerXidLock				(&MainLWLockArray[31].lock)
#define SyncRepLock					(&MainLWLockArray[32].lock)
#define BackgroundWorkerLock		(&MainLWLockArray[33].lock)
#define DynamicSharedMemoryControlLock		(&MainLWLockArray[34].lock)
#define AutoFileLock				(&MainLWLockArray[35].lock)
#define ReplicationSlotAllocationLock	(&MainLWLockArray[36].lock)
#define ReplicationSlotControlLock		(&MainLWLockArray[37].lock)
#define CommitTsControlLock			(&MainLWLockArray[38].lock)
#define CommitTsLock				(&MainLWLockArray[39].lock)*/

#[cfg(test)]
mod tests {
    use super::{
        LWLock,
    };

    #[test]
    fn test_sh_mem_lock() {
        static SH_MEM_INDEX_LOCK: LWLock<()> = lwlock_init!(());

        spin_lock_acquire!(guard = SH_MEM_INDEX_LOCK.mutex, {
            println!("{}", guard.deref().1.header);
        })
    }

    /*#[test]
    fn minimal_tuple_padding() {
        let size = mem::size_of(LWLock);
        assert_eq!(LWLOCK_PADDED_SIZE, mem::size_of::<LWLock>() + LWLOCK_PADDING);
    }*/
}
