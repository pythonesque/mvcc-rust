// Hardware-dependent spin locks.

use std::cmp;
use std::cell::Cell;
use std::time::Duration;
use std::io::{IoResult, Timer};
use std::kinds::marker;
use std::rand::{mod, Closed01, Rand, Rng};
use std::rt;

#[cfg(target_arch = "x86_64")] type slock = u8;

#[repr(C)]
pub struct SLock {
    lock: ::std::cell::UnsafeCell<slock>,
    nocopy: marker::NoCopy
}

const DEFAULT_SPINS_PER_DELAY: u32 = 100;
thread_local! {
    static SPINS_PER_DELAY: Cell<u32> = Cell::new(DEFAULT_SPINS_PER_DELAY)
}

pub fn set_spins_per_delay(shared_spins_per_delay: u32) {
    SPINS_PER_DELAY.with(|spins_per_delay| spins_per_delay.set(shared_spins_per_delay) );
}

pub fn update_spins_per_delay(shared_spins_per_delay: u32) -> u32 {
    SPINS_PER_DELAY.with(|spins_per_delay|
                         (shared_spins_per_delay * 15 + spins_per_delay.get()) / 16 )
}

impl SLock {
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    fn tas(&self) -> bool {
        unsafe {
            ::std::intrinsics::atomic_xchg(self.lock.get(), 1) != 0
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    fn tas_spin(&self) -> bool {
        unsafe {
            *self.lock.get() != 0 || self.tas()
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    fn spin_delay() {
        unsafe {
            asm!("rep; nop")
        }
    }

    // Default definitions -- override these as needed

    #[inline(always)]
    pub fn lock_(&self, file_line: &(&'static str, uint)) -> IoResult<u32> {
        if self.tas() {
            self.lock(file_line)
        } else {
            Ok(0)
        }
    }

    #[inline(always)]
    pub fn free_(&self) -> bool {
        unsafe {
            ::std::intrinsics::atomic_load_acq(self.lock.get() as *const slock) == 0
        }
    }

    #[inline(always)]
    pub fn unlock_(&self) {
        unsafe {
            ::std::intrinsics::atomic_store_rel(self.lock.get(), 0)
        }
    }

    #[inline(always)]
    pub fn init_() -> SLock {
        SLock {
            lock: ::std::cell::UnsafeCell::new(0),
            nocopy: marker::NoCopy,
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[inline(always)]
    fn spin_delay() { }

    //#[cold] #[inline(never)]
    fn lock_stuck(file_line: &(&'static str, uint)) {
        rt::begin_unwind("Stuck spinlock detected", file_line);
    }

    fn lock(&self, file_line: &(&'static str, uint)) -> IoResult<u32> {
        const MIN_SPINS_PER_DELAY: u32 = 10;
        const MAX_SPINS_PER_DELAY: u32 = 1000;
        const NUM_DELAYS: u32 = 1000;
        const MIN_DELAY_USEC: u32 = 1000;
        const MAX_DELAY_USEC: u32 = 1000000;
        SPINS_PER_DELAY.with( |spins_per_delay_| {
            let mut spins = 0;
            let mut delays = 0;
            let mut cur_delay = 0;
            let mut rng = None;
            let mut timer = None;

            let spins_per_delay = spins_per_delay_.get();

            while self.tas_spin() {
                // CPU-specific delay each time through the loop
                SLock::spin_delay();

                // Block the process every spins_per_delay tries
                spins += 1;
                if spins >= spins_per_delay {
                    delays += 1;
                    if delays > NUM_DELAYS {
                        SLock::lock_stuck(file_line);
                    }

                    if cur_delay == 0 { // first time to delay?
                        cur_delay = MIN_DELAY_USEC;
                    }

                    let duration = Duration::microseconds(cur_delay as i64);
                    match timer {
                        None => {
                            let mut timer_ = try!(Timer::new());
                            timer_.sleep(duration);
                            timer = Some(timer_);
                        }
                        Some(ref mut timer) => timer.sleep(duration),
                    };
                    //::TIMER.with( |timer|  );
                    // increase delay by a random fraction between 1X and 2X
                    cur_delay += (cur_delay as f64 * match rng {
                        None => {
                            let mut rng_ = rand::task_rng();
                            let frac = rng_.gen::<Closed01<f64>>().0;
                            rng = Some(rng_);
                            frac
                        },
                        Some(ref mut rng) => rng.gen::<Closed01<f64>>().0,
                    }) as u32;
                    // wrap back to minimum delay when maximum is exceeded
                    if cur_delay > MAX_DELAY_USEC {
                        cur_delay = MIN_DELAY_USEC;
                    }

                    spins = 0;
                }
            }

            if cur_delay == 0 {
                // we never had to delay
                if spins_per_delay < MAX_SPINS_PER_DELAY {
                    spins_per_delay_.set(cmp::min(spins_per_delay + 100, MAX_SPINS_PER_DELAY));
                }
            } else {
                if spins_per_delay > MIN_SPINS_PER_DELAY {
                    spins_per_delay_.set(cmp::max(spins_per_delay - 1, MIN_SPINS_PER_DELAY));
                }
            }
            Ok(delays)
        } )
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "long-tests")]
    #[test]
    pub fn test_lock() {
        use std::sync::Arc;
        use super::SLock;

        let s_lock = Arc::new(SLock::init_());

        assert!(s_lock.free_());

        static FILE_LINE_1: &'static (&'static str, uint) = &(file!(), line!());
        s_lock.lock_(FILE_LINE_1);

        assert!(!s_lock.free_());

        s_lock.unlock_();

        assert!(s_lock.free_());

        static FILE_LINE_2: &'static (&'static str, uint) = &(file!(), line!());
        s_lock.lock_(FILE_LINE_2);

        assert!(!s_lock.free_());

        assert!(::std::task::try(proc() {
            static FILE_LINE_3: &'static (&'static str, uint) = &(file!(), line!());
            s_lock.lock_(FILE_LINE_3);
        }).is_err());
    }
}
