use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ringbuffer::RingBuffer;

fn bench_ring_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("ringbuffer_vs_mpsc");

    group.sample_size(50);

    const ITEMS: usize = 3000;

    group.bench_function("RingBuffer (SPSC lock-free)", |b| {
        b.iter(|| {
            let rb = RingBuffer::<usize, 1024>::new();
            let mut count = 0;

            for i in 0..ITEMS {
                while rb.push(black_box(i)).is_err() {
                    std::hint::spin_loop();
                }
                // pop immediately (this keeps the buffer from filling)
                if rb.pop().is_some() {
                    count += 1;
                }
            }

            while rb.pop().is_some() {
                count += 1;
            }
            black_box(count);
        })
    });

    group.bench_function("std::sync::mpsc", |b| {
        b.iter(|| {
            let (tx, rx) = std::sync::mpsc::channel::<usize>();
            let mut count = 0;

            for i in 0..ITEMS {
                let _ = tx.send(black_box(i));
                if rx.try_recv().is_ok() {
                    count += 1;
                }
            }

            while rx.try_recv().is_ok() {
                count += 1;
            }
            black_box(count);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_ring_buffer);
criterion_main!(benches);
