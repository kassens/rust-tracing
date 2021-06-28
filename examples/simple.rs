use std::fs;
use std::time::{Duration, Instant};

use tracing_timeline::frame;
use tracing_timeline::profiler::{Profiler, Span};

fn main() {
    let profiles_builder = Profiler::new();
    let mut thread = profiles_builder.thread("main");
    let mut s = thread.span(frame!("main"));

    query_files(&mut s);

    build_projects(&mut s);

    write_result(&mut s);

    fs::write(
        "profile.json",
        serde_json::to_string_pretty(&profiles_builder.complete()).unwrap(),
    )
    .unwrap();
}

fn build_projects(span: &mut Span<'_>) {
    let mut s = span.span(frame!("build_projects"));
    build_project(&mut s, 1);
    build_project(&mut s, 2);
}

fn build_project(span: &mut Span<'_>, secs: u32) {
    let mut s = span.span(frame!("build_project"));
    fib(&mut s, secs);
}

fn query_files(span: &mut Span<'_>) {
    let _s = span.span(frame!("query_files"));
    spin(Duration::from_millis(100));
}

fn write_result(span: &mut Span<'_>) {
    let _s = span.span(frame!("write_result"));
    spin(Duration::from_millis(300));
}

fn spin(duration: Duration) {
    // let _s = Span::new("spin");
    let t = Instant::now();
    while t.elapsed() < duration {}
}

fn fib(parent: &mut Span<'_>, n: u32) -> u32 {
    let mut s = parent.span(frame!("fib"));
    spin(Duration::from_millis(20));
    if n < 2 {
        return n;
    }
    return fib(&mut s, n - 1) + fib(&mut s, n - 2);
}
