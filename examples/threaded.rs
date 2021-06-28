use std::fs;
use std::time::{Duration, Instant};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing_timeline::frame;
use tracing_timeline::profiler::{Profiler, Span};

fn main() {
    let profiles_builder = Profiler::new();
    let mut thread = profiles_builder.thread("main");
    let mut main_span = thread.span(frame!("main"));

    query_files(&mut main_span);

    build_projects(&mut main_span);

    write_result(&mut main_span);

    drop(main_span);
    thread.complete();

    fs::write(
        "profile.json",
        serde_json::to_string_pretty(&profiles_builder.complete()).unwrap(),
    )
    .unwrap();
}

fn build_projects(s: &mut Span<'_>) {
    let s = s.span(frame!("build_projects"));

    [3, 4, 5].par_iter().for_each(|i| {
        let mut p = s.fork(&format_args!("build_project_{}", i));
        let mut s = p.span(frame!("loop"));
        build_project(&mut s, *i);
        drop(s);
        p.complete();
    });
}

fn build_project(p: &mut Span<'_>, n: u32) {
    let mut s = p.span(frame!("build_project"));

    fib(&mut s, n);
}

fn query_files(p: &mut Span<'_>) {
    let _s = p.span(frame!("query_files"));
    spin(Duration::from_millis(100));
}

fn write_result(p: &mut Span<'_>) {
    let _s = p.span(frame!("write_result"));
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
