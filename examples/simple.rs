use std::fs;
use std::time::{Duration, Instant};

use tracing_timeline::frame;
use tracing_timeline::profiler::{Profiler, ProfilerScope};

fn main() {
    let profiles_builder = Profiler::new();

    let mut profiler = profiles_builder.profile("main");

    query_files(&mut profiler);

    build_projects(&mut profiler);

    write_result(&mut profiler);

    fs::write(
        "profile.json",
        serde_json::to_string_pretty(&profiler.complete()).unwrap(),
    )
    .unwrap();
}

fn build_projects(p: &mut impl ProfilerScope) {
    let mut s = p.span(frame!("build_projects"));
    build_project(&mut s, 1);
    build_project(&mut s, 2);
}

fn build_project(p: &mut impl ProfilerScope, secs: u32) {
    let mut s = p.span(frame!("build_project"));
    fib(&mut s, secs);
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
