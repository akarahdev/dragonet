#![cfg(test)]

use std::sync::Mutex;
use crate::Runtime;



#[test]
fn basic_runtime() {
    let mut rt = Runtime::new();
    rt.spawn(async {
        runtime_example().await;
    });
    rt.spawn(async {
        example_2().await;
    });
    rt.run();
}

async fn runtime_example() {
    let v = futures::join!(compute_something(), compute_something_else());
    assert_eq!(v.0, 10);
    assert_eq!(v.1, 15);
    println!("v: {:?}", v);
}

async fn example_2() {
    let v = futures::join!(compute_something(), compute_something_else());
    assert_eq!(v.0, 10);
    assert_eq!(v.1, 15);
    println!("v2: {:?}", v);
}

async fn compute_something() -> i32 {
    10
}

async fn compute_something_else() -> i32 {
    15
}