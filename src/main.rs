use ringbuffer::RingBuffer;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

fn main() {
    let rb = Arc::new(RingBuffer::<i32, 8>::new());
    let done = Arc::new(AtomicBool::new(false));

    let producer = {
        let rb = rb.clone();
        let done = done.clone();

        thread::spawn(move || {
            for i in 0..100 {
                // retry if full
                loop {
                    if rb.push(i).is_ok() {
                        println!("Pushed {}", i);
                        break;
                    }
                    std::hint::spin_loop();
                }
            }

            done.store(true, Ordering::Release);
        })
    };

    let consumer = {
        let rb = rb.clone();
        let done = done.clone();

        thread::spawn(move || {
            loop {
                if let Some(val) = rb.pop() {
                    println!("Popped {}", val);
                } else if done.load(Ordering::Acquire) {
                    break;
                } else {
                    std::hint::spin_loop();
                }
            }
        })
    };

    producer.join().unwrap();
    consumer.join().unwrap();
}
