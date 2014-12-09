#![macro_escape]

use self::LWLockMode::*;

use s_lock::{
    SpinLock
};
use process;

use std::thread_local::Key;
use std::fmt;

pub struct LWLockHeader {
    /// T if ok to release waiters
    release_ok: bool,
    /// has an exclusive holder?
    exclusive: bool,
    /// # of shared holders (0..MaxBackends)
    shared: u32,
    //tranche: u32, /// tranche ID
    /// head of list of waiting Procs
    head: Option<&'static Key<process::Proc>>,
    /// tail of list of waiting Procss
    tail: Option<&'static Key<process::Proc>>,
    // tail is undefined when head is NULL
}

impl fmt::Show for LWLockHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LWLockHeader {{ \
                  release_ok: {}, \
                  exclusive: {}, \
                  shared: {}, \
                  head: {}, \
                  tail: {} }}",
                  self.release_ok,
                  self.exclusive,
                  self.shared,
                  self.head.map( |p| p as *const _),
                  self.tail.map( |p| p as *const _),
        )
    }
}

#[deriving(Show)]
#[repr(u8)]
pub enum LWLockMode {
    Exclusive,
    Shared,
    WaitUntilFree,
}

#[deriving(Show)]
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
                        exclusive: false,
                        shared: 0,
                        head: None,
                        tail: None,
                    },
                    data: $data,
                }
            )
        }
    )
)

impl<T> LWLock<T> where T: Send {
    pub fn new(data: T) -> LWLock<T> {
        lwlock_init!(data)
    }

    pub fn acquire(&self, mode: LWLockMode) -> bool {
        self.acquire_common(mode)
    }

    fn acquire_common(&self, mode: LWLockMode) -> bool {
        process::MY_PROC.with( |thread| {
            let mut retry = false;
            let mut result = true;
            let mut extra_waits = 0u32;

            // Assert(!(proc == NULL && IsUnderPostmaster));

            // Ensure we will have room to remember the lock
            // if (num_held_lwlocks >= MAX_SIMUL_LWLOCKS) ERROR

            // Lock out cancel/die interrupts until we exit the code section protected
            // by the LWLock.  This ensures that interrupts will not interfere with
            // manipulations of data structures in shared memory.
            // HOLD_INTERRUPTS

            // Loop here to try to acquire lock after each time we are signaled by
            // LWLockRelease.
            let mut guard;
            loop {
                let must_wait;

                // Acquire mutex.  Time spent holding mutex should be short!
                static FILE_LINE: &'static (&'static str, uint) = &(file!(), line!() + 1);
                guard = self.mutex.acquire_guard(FILE_LINE);
                {
                    let mut lock = guard.deref_mut().1;
                    // If retrying, allow LWLockRelease to release waiters again
                    if retry {
                        lock.header.release_ok = true;
                    }

                    // If I can get the lock, do so quickly.
                    match mode {
                        Exclusive => {
                            if !lock.header.exclusive && lock.header.shared == 0 {
                                lock.header.exclusive = true;
                                must_wait = false
                            } else {
                                must_wait = true
                            }
                        },
                        _ => {
                            if !lock.header.exclusive {
                                lock.header.shared += 1;
                                must_wait = false
                            } else {
                                must_wait = true
                            }
                        }
                    }

                    if !must_wait {
                        break
                    }

                    // Add myself to wait queue.
                    thread.lw_waiting.set(true);
                    thread.lw_wait_mode.set(mode);
                    thread.lw_wait_link.set(None);
                    match lock.header.head {
                        Some(_) => {
                            // Note: we are assuming that tail was set correctly!
                            match lock.header.tail {
                                Some(tail) => tail.with( |p| p.lw_wait_link.set(Some(&process::MY_PROC))),
                                None => unreachable!(),
                            }
                        },
                        None => lock.header.head = Some(&process::MY_PROC)
                    }
                    //*lock.header.tail.as_ref().unwrap() = ;
                    lock.header.tail = Some(&process::MY_PROC);
                }

                // Can release the mutex now
                drop(guard);

                // Wait until awakened.
                loop {
                    // TODO: disable interrupts
                    thread.sem.acquire();
                    if !thread.lw_waiting.get() {
                        break
                    }
                    extra_waits += 1;
                }

                // Now loop back and try to acquire lock again
                retry = true;
                result = false;
            }

            // If there's a variable associated with this lock, initialize it
            // TODO: add this

            // We are done updating shared state of the lock itself.
            drop(guard);

            // Add lock to list of locks held by this backend
            // held_lwlocks[num_held_lwlocks++] = lock;

            // Fix the process wait semaphore's count for any absorbed wakeups.
            while extra_waits > 0 {
                extra_waits -= 1;
                thread.sem.release();
            }

            result
        })
    }

    pub unsafe fn release(&self) {
        // Remove lock from list of locks held.  Usually, but not always, it will
        // be the latest-acquired lock; so search array backwards.

    }
}

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
        assert_eq!(LWLOCK_PADDED_SIZE, mem::size_of::<LWLock>() + LWLOCK_PADDING)
    }*/
}
