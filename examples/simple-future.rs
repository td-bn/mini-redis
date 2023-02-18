use std::{
    future::Future,
    time::{Duration, Instant}, task::Poll,
};

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if Instant::now() > self.when {
            println!("Hello, World!");
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_secs(5);
    let future = Delay { when };

    let out = future.await;
    println!("{out}");
}

