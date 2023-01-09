/// 异步部分的所有代码都在 test_async 和 task 文件夹中
/// test_async 中主要是测试代码 和 初始化代码
/// task/mod.rs 中包含了一个简单的执行器
/// task/timer.rs 中包含了一个 timer example
/// task/interrupt_wakeups 中包含了 Waker 的基本信息

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use core::task::{Context, Poll};
use futures_util::Stream;
use spin::Lazy;
use crate::mutex::Mutex;
use crate::sbi::shutdown;
use crate::task::Executor;
use crate::task::interrupt_wakeups::interrupt_wake;
use crate::task::Task;

static TICKS: AtomicUsize = AtomicUsize::new(0);

async fn async_number() -> u32 {
    42
}

struct WaitValue(pub usize);

impl Future for WaitValue {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        // WAKER.register(&cx.waker());
        let current_ticks = TICKS.load(Ordering::Acquire);
        // println!("read_current ticks: {}", current_ticks);
        if current_ticks > self.0 {
            Poll::Ready(())
        } else {
            // pending 之后是不会在满足条件的时候直接执行的  需要使用 `interrupt_wake(cx.waker.clone())
            // 加入队列后等待执行
            interrupt_wake(cx.waker().clone());
            Poll::Pending
        }
    }
}


async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

fn wait_value(value: usize) -> impl Future<Output = ()> {
    WaitValue(value)
}

async fn wait_for_t1() {
    WaitValue(1000).await;
    println!("Hello WOrld!");
}

pub fn add_t1() {
    let value = TICKS.load(Ordering::SeqCst);
    if value <= 1000 {
        TICKS.fetch_add(1, Ordering::SeqCst);
        // println!("add value: {}", value);
    }
}

pub fn init() -> ! {
    // […] initialization routines, including `init_heap`
    crate::task::init();
    let mut executor = Executor::new();
    executor.spawn(wait_for_t1());
    executor.spawn(crate::task::timer::timer_task());
    executor.spawn(example_task());
    executor.spawn(example_task());
    // executor.spawn(add_t1());
    executor.run();

    // […] test_main, "it did not crash" message, hlt_loop
    shutdown()
}