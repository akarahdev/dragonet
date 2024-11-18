#![cfg(test)]

use crate::Runtime;

#[test]
fn basic_runtime() {
    let mut rt = Runtime::new();
    rt.spawn(async {
        runtime_example().await;
    });
    rt.run();
}

async fn runtime_example() {
    let num = compute_something().await;
    assert_eq!(num, 10);
}

async fn compute_something() -> i32 {
    10
}