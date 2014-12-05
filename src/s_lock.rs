// Hardware-dependent spin locks.

use std::kinds::marker;

#[repr(C)]
struct SLock {
    #[cfg(target_arch = "x86_64")] lock: ::std::cell::UnsafeCell<u8>,
    nocopy: marker::NoCopy
}

#[cfg(target_arch = "x86_64")]
impl SLock {
    #[inline(always)]
    fn tas(&self) -> u8 {
        unsafe {
            ::std::intrinsics::atomic_xchg(self.lock.get(), 1)
        }
    }

    #[inline(always)]
    fn spin_delay() {
        unsafe {
            asm!("rep; nop")
        }
    }

    #[inline(always)]
    // Works on x86 because why not
    pub fn unlock(&self) {
        unsafe {
            *self.lock.get() = 0;
        }
    }
}
