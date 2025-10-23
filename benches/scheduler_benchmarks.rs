use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use taskline::{Scheduler, Task};
use std::time::Duration;

fn bench_task_creation(c: &mut Criterion) {
    c.bench_function("task_creation", |b| {
        b.iter(|| {
            Task::new(|| async { Ok(()) })
        });
    });
}

fn bench_task_execution(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("task_execution", |b| {
        let task = Task::new(|| async { Ok(()) });

        b.to_async(&runtime).iter(|| async {
            task.execute().await.unwrap();
        });
    });
}

fn bench_scheduler_add_tasks(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler_add_tasks");

    for task_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(task_count),
            task_count,
            |b, &count| {
                b.iter(|| {
                    let scheduler = Scheduler::new();
                    for i in 0..count {
                        let task = Task::new(|| async { Ok(()) })
                            .with_name(format!("Task {}", i));
                        scheduler.add("* * * * *", task).unwrap();
                    }
                    black_box(scheduler);
                });
            },
        );
    }

    group.finish();
}

fn bench_scheduler_lifecycle(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("scheduler_lifecycle", |b| {
        b.to_async(&runtime).iter(|| async {
            let scheduler = Scheduler::new();
            let task = Task::new(|| async { Ok(()) });
            scheduler.add("* * * * *", task).unwrap();

            scheduler.start().await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
            scheduler.stop().await.unwrap();
        });
    });
}

fn bench_concurrent_task_execution(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_tasks");

    for task_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(task_count),
            task_count,
            |b, &count| {
                b.to_async(&runtime).iter(|| async move {
                    let mut handles = vec![];

                    for _ in 0..count {
                        let task = Task::new(|| async {
                            tokio::time::sleep(Duration::from_micros(100)).await;
                            Ok(())
                        });

                        let handle = tokio::spawn(async move {
                            task.execute().await
                        });

                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.await.unwrap().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_task_creation,
    bench_task_execution,
    bench_scheduler_add_tasks,
    bench_scheduler_lifecycle,
    bench_concurrent_task_execution
);
criterion_main!(benches);
