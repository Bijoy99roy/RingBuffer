
use ringbuffer::RingBuffer; 
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

fn main() {
    // 8 capacity buffer
    let rb = Arc::new(RingBuffer::<i32, 8>::new());
    let done = Arc::new(AtomicBool::new(false));

    let mut producers = vec![];

    for producer_id in 0..3 {
        let rb = rb.clone();
        
        producers.push(thread::spawn(move || {
            for i in 0..10 {
                // Create a unique message per producer 
                let msg = (producer_id * 100) + i;
                
                // Retry if full
                loop {
                    if rb.push(msg).is_ok() {
                        println!("Producer {} pushed {}", producer_id, msg);
                        break;
                    }
                    std::hint::spin_loop();
                }
            }
        }));
    }

    let consumer = {
        let rb = rb.clone();
        let done = done.clone();

        thread::spawn(move || {
            loop {
                if let Some(val) = rb.pop() {
                    println!("Popped {}", val);
                } else if done.load(Ordering::Acquire) {
                    // Double-check the buffer is actually empty after done is set to prevent dropping final items
                    if let Some(val) = rb.pop() {
                        println!("Popped {}", val);
                        continue;
                    }
                    break;
                } else {
                    std::hint::spin_loop();
                }
            }
        })
    };

    for p in producers {
        p.join().unwrap();
    }

    done.store(true, Ordering::Release);
    consumer.join().unwrap();
}