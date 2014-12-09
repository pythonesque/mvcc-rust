/// Per-thread shared memory data structures

use lwlock::{
    LWLockMode,
};

use std::cell::Cell;
use std::fmt;
use std::sync::Semaphore;
use std::thread_local::Key;

#[repr(C)]
pub struct Proc {
    // SHM_QUEUE   links,          /* list link if process is in a list */

    /// ONE semaphore to sleep on
    pub sem: Semaphore,

    // Info about LWLock the process is currently waiting for, if any.
    /// true if waiting for an LW lock
    pub lw_waiting: Cell<bool>,
    /// lwlock mode being waited for
    pub lw_wait_mode: Cell<LWLockMode>,
    /// next waiter for same LW lock
    pub lw_wait_link: Cell<Option<&'static Key<Proc>>>,
}

impl fmt::Show for Proc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Proc {{ \
                  lw_waiting: {}, \
                  lw_wait_mode: {}, \
                  lw_wait_link: {} }}",
                  self.lw_waiting,
                  self.lw_wait_mode,
                  self.lw_wait_link.get().map( |p| p as *const _),
        )
    }
}

thread_local!(pub static MY_PROC: Proc = Proc {
    sem: Semaphore::new(1),
    lw_waiting: Cell::new(false),
    lw_wait_mode: Cell::new(LWLockMode::WaitUntilFree),
    lw_wait_link: Cell::new(None),
})
