// Hardware-dependent spin locks.

use std::cmp;
use std::cell::UnsafeCell;
use std::io::{IoResult, Timer};
use std::kinds::marker;
use std::rand::{mod, Closed01, Rng};
use std::rt;
use std::time::Duration;

#[cfg(target_arch = "x86_64")] type SLock_ = u8;

#[repr(C)]
pub struct SLock {
    lock: UnsafeCell<SLock_>,
    nocopy: marker::NoCopy
}

const DEFAULT_SPINS_PER_DELAY: u32 = 100;
#[thread_local] static mut SPINS_PER_DELAY: u32 = DEFAULT_SPINS_PER_DELAY;

pub fn set_spins_per_delay(shared_spins_per_delay: u32) {
    unsafe {
        SPINS_PER_DELAY = shared_spins_per_delay;
    }
}

pub fn update_spins_per_delay(shared_spins_per_delay: u32) -> u32 {
    unsafe {
        let spins_per_delay = SPINS_PER_DELAY;
        (shared_spins_per_delay * 15 + spins_per_delay) / 16
    }
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
            ::std::intrinsics::atomic_load_acq(self.lock.get() as *const SLock_) == 0
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
            lock: UnsafeCell::new(0),
            nocopy: marker::NoCopy,
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    #[inline(always)]
    fn spin_delay() { }

    #[cold] #[inline(never)]
    fn lock_stuck(file_line: &(&'static str, uint)) {
        rt::begin_unwind("Stuck spinlock detected", file_line);
    }

    fn lock(&self, file_line: &(&'static str, uint)) -> IoResult<u32> {
        const MIN_SPINS_PER_DELAY: u32 = 10;
        const MAX_SPINS_PER_DELAY: u32 = 1000;
        const NUM_DELAYS: u32 = 1000;
        const MIN_DELAY_USEC: u32 = 1000;
        const MAX_DELAY_USEC: u32 = 1000000;
        {
            let mut spins = 0;
            let mut delays = 0;
            let mut cur_delay = 0;
            let mut rng = None;
            let mut timer = None;

            let spins_per_delay = unsafe { SPINS_PER_DELAY };

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
                    unsafe {
                        SPINS_PER_DELAY = cmp::min(spins_per_delay + 100, MAX_SPINS_PER_DELAY);
                    }
                }
            } else {
                if spins_per_delay > MIN_SPINS_PER_DELAY {
                    unsafe {
                        SPINS_PER_DELAY = cmp::max(spins_per_delay - 1, MIN_SPINS_PER_DELAY);
                    }
                }
            }
            Ok(delays)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SLock;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_lock() {
        let s_lock = SLock::init_();

        assert!(s_lock.free_());

        static FILE_LINE: &'static (&'static str, uint) = &(file!(), line!() + 1);
        assert!(s_lock.lock_(FILE_LINE).is_ok());
        assert!(!s_lock.free_());

        s_lock.unlock_();
        assert!(s_lock.free_());
    }

    #[test]
    #[cfg(feature = "long-tests")]
    fn test_lock_long() {
        let s_lock = SLock::init_();

        assert!(s_lock.free_());

        static FILE_LINE_1: &'static (&'static str, uint) = &(file!(), line!() + 1);
        assert!(s_lock.lock_(FILE_LINE_1).is_ok());
        assert!(!s_lock.free_());

        assert!(::std::task::try(proc() {
            static FILE_LINE_2: &'static (&'static str, uint) = &(file!(), line!() + 1);
            assert!(s_lock.lock_(FILE_LINE_2).is_ok());
        }).is_err());
    }

    #[bench]
    fn bench_lock_unlock_mutex(b: &mut ::test::Bencher) {
        let mutex = Mutex::new(());
        b.iter( || {
            let _guard = mutex.lock();
        })
    }

    #[bench]
    fn bench_lock(b: &mut ::test::Bencher) {
        b.iter( || {
            let s_lock = SLock::init_();
            static FILE_LINE: &'static (&'static str, uint) = &(file!(), line!() + 1);
            assert!(s_lock.lock_(FILE_LINE).is_ok());
        })
    }

    #[bench]
    fn bench_unlock(b: &mut ::test::Bencher) {
        let s_lock = SLock::init_();
        b.iter( || {
            s_lock.unlock_();
        })
    }

    #[bench]
    fn bench_lock_unlock(b: &mut ::test::Bencher) {
        let s_lock = SLock::init_();

        b.bytes = 1; // One byte modified per iteration

        b.iter( || {
            static FILE_LINE: &'static (&'static str, uint) = &(file!(), line!() + 1);
            assert!(s_lock.lock_(FILE_LINE).is_ok());
            s_lock.unlock_();
        })
    }

    #[bench]
    fn bench_lock_unlock_contended(b: &mut ::test::Bencher) {
        use std::sync::atomic;

        b.bytes = 1; // One byte modified per iteration

        let s_lock = Arc::new(SLock::init_());
        let done = Arc::new(atomic::AtomicBool::new(false));
        let done_ = done.clone();
        let s_lock_ = s_lock.clone();
        spawn(proc() {
            while !done_.load(atomic::Ordering::Relaxed) {
                static FILE_LINE_2: &'static (&'static str, uint) = &(file!(), line!() + 1);
                assert!(s_lock_.lock_(FILE_LINE_2).is_ok());
                s_lock_.unlock_();
            }
        });
        b.iter( || {
            static FILE_LINE_1: &'static (&'static str, uint) = &(file!(), line!() + 1);
            assert!(s_lock.lock_(FILE_LINE_1).is_ok());
            s_lock.unlock_();
        });
        done.store(true, atomic::Ordering::Relaxed);
    }
}
