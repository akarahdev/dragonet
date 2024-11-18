#![cfg(test)]

use std::sync::Mutex;
use crate::Runtime;



#[test]
fn basic_runtime() {
    Runtime::spawn(async {
        runtime_example().await;
    });
    Runtime::run();
}

async fn runtime_example() {
    let num = compute_something().await;
    assert_eq!(num, 10);
}

async fn compute_something() -> i32 {
    10
}