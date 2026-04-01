use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

#[repr(align(64))]
struct CachePadded<T>(T);

pub struct RingBuffer<T, const CAP: usize> {
    head: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
    buffer: UnsafeCell<[MaybeUninit<T>; CAP]>,
}

impl<T, const CAP: usize> RingBuffer<T, CAP> {
    pub fn new() -> Self {
        assert!(
            CAP.is_power_of_two(),
            "CAP must be power of 2 for fast masking based operation"
        );
        Self {
            head: CachePadded(AtomicUsize::new(0)),
            tail: CachePadded(AtomicUsize::new(0)),
            buffer: UnsafeCell::new(std::array::from_fn(|_| MaybeUninit::uninit())),
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        let mut head = self.head.0.load(Ordering::Relaxed);

        loop {
            let tail = self.tail.0.load(Ordering::Acquire);

            if head.wrapping_sub(tail) == CAP {
                return Err(value);
            }

            match self.head.0.compare_exchange(
                head,
                head.wrapping_add(1),
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Since regular modulo operation is slow and expensive
                    // So, instead of doing head % CAP. We are using  head & (CAP - 1)
                    // Bitwise operation since it's faster than normal arithmatic operation
                    // CAP needs to be power of 2 for this operation since it works only for a value that is power of 2
                    unsafe {
                        let buffer = &mut *self.buffer.get();
                        buffer[head & (CAP - 1)].write(value);
                    }
                    return Ok(());
                }
                Err(current_head) => {
                    // Since lost race to another producer, retry with new head
                    head = current_head
                }
            }

            self.head.0.store(head.wrapping_add(1), Ordering::Release);
        }
    }

    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.0.load(Ordering::Relaxed);
        let head = self.head.0.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let value = unsafe {
            let buffer = &mut *self.buffer.get();
            buffer[tail & (CAP - 1)].assume_init_read()
        };

        self.tail.0.store(tail.wrapping_add(1), Ordering::Release);

        Some(value)
    }
}

unsafe impl<T: Send, const CAP: usize> Sync for RingBuffer<T, CAP> {}
unsafe impl<T: Send, const CAP: usize> Send for RingBuffer<T, CAP> {}

impl<T, const CAP: usize> Drop for RingBuffer<T, CAP> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}
