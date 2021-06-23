use std::fs;
use std::time::{Duration, Instant};

use tracing_timeline::frame;
use tracing_timeline::profiler::{Profiler, ProfilerScope, Thread};

fn main() {
    let profiles_builder = Profiler::new();
    let mut profiler = profiles_builder.profile("main");

    query_files(&mut profiler);

    build_projects(&mut profiler, &profiles_builder);

    write_result(&mut profiler);

    profiler.complete();

    fs::write(
        "profile.json",
        serde_json::to_string_pretty(&profiles_builder.complete()).unwrap(),
    )
    .unwrap();
}

fn build_projects(p: &mut impl ProfilerScope, profiles_builder: &Profiler) {
    let _s = p.span(frame!("build_projects"));

    let join_handles = [3, 4, 5]
        .iter()
        .map(|&i| {
            let mut p: Thread = profiles_builder.profile(format!("build_project_{}", i));
            std::thread::spawn(move || {
                build_project(&mut p, i);
                p.complete();
            })
        })
        .collect::<Vec<_>>();

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }
}

fn build_project(p: &mut impl ProfilerScope, n: u32) {
    let mut s = p.span(frame!("build_project"));
    fib(&mut s, n);
}

fn query_files(p: &mut impl ProfilerScope) {
    let _s = p.span(frame!("query_files"));
    spin(Duration::from_millis(100));
}

fn write_result(p: &mut impl ProfilerScope) {
    let _s = p.span(frame!("write_result"));
    spin(Duration::from_millis(300));
}

fn spin(duration: Duration) {
    // let _s = Span::new("spin");
    let t = Instant::now();
    while t.elapsed() < duration {}
}

fn fib(parent: &mut impl ProfilerScope, n: u32) -> u32 {
    let mut s = parent.span(frame!("fib"));
    spin(Duration::from_millis(20));
    if n < 2 {
        return n;
    }
    return fib(&mut s, n - 1) + fib(&mut s, n - 2);
}
