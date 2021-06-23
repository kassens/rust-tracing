use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::frame::get_frames_clone;
use crate::speedscope::{
    Event, EventedProfile, File, FileShared, FrameIndex, ProfileTypeEvented, SpeedscopeSchemaUrl,
    Value, ValueUnit,
};
use std::sync::Arc;
use std::time::Instant;

pub struct Profiler {
    start_value: Value,
    threads: Arc<Mutex<Vec<EventedProfile>>>,
}
impl Profiler {
    pub fn new() -> Self {
        Self {
            start_value: elapsed(),
            threads: Default::default(),
        }
    }
    pub fn profile(&self, name: impl Into<String>) -> Thread {
        Thread::new(self, name.into())
    }
    pub fn complete(self) -> File {
        let end_value = elapsed();
        let mut threads = Arc::try_unwrap(self.threads)
            .unwrap_or_else(|_| panic!())
            .into_inner();
        threads.sort_by_key(|t| t.start_value);

        // Set uniform start/end values to help understand relative time.
        // Otherwise, everything is visualized to start at the beginning.
        for t in &mut threads {
            t.start_value = self.start_value;
            t.end_value = end_value;
        }
        File {
            schema: SpeedscopeSchemaUrl,
            shared: FileShared {
                frames: get_frames_clone(),
            },
            profiles: threads,
            name: None,
            active_profile_index: None,
            exporter: None,
        }
    }
}

pub struct Thread {
    name: String,
    profiles: Arc<Mutex<Vec<EventedProfile>>>,
    start_value: Value,
    events: Vec<Event>,
}
impl Thread {
    fn new(profiles_builder: &Profiler, name: String) -> Self {
        Self {
            name,
            profiles: Arc::clone(&profiles_builder.threads),
            start_value: elapsed(),
            events: Vec::new(),
        }
    }
    pub fn complete(self) {
        self.profiles.lock().push(EventedProfile {
            type_: ProfileTypeEvented,
            name: self.name,
            unit: ValueUnit::Microseconds,
            start_value: self.start_value,
            end_value: elapsed(),
            events: self.events,
        });
    }
}

pub trait ProfilerScope {
    fn span<'a>(&'a mut self, frame: FrameIndex) -> Span<'a>;
}
impl ProfilerScope for Thread {
    fn span<'a>(&'a mut self, frame: FrameIndex) -> Span<'a> {
        Span::new(&mut self.events, frame)
    }
}
impl ProfilerScope for Span<'_> {
    fn span<'a>(&'a mut self, frame: FrameIndex) -> Span<'a> {
        Span::new(&mut self.events, frame)
    }
}

fn elapsed() -> Value {
    static START: Lazy<Instant> = Lazy::new(Instant::now);
    Value::new(START.elapsed().as_micros() as u64)
}

pub struct Span<'a> {
    events: &'a mut Vec<Event>,
    frame: FrameIndex,
}
impl<'a> Span<'a> {
    fn new(events: &'a mut Vec<Event>, frame: FrameIndex) -> Self {
        events.push(Event::OpenFrame {
            at: elapsed(),
            frame,
        });
        Self { events, frame }
    }
}
impl<'a> Drop for Span<'a> {
    fn drop(&mut self) {
        self.events.push(Event::CloseFrame {
            at: elapsed(),
            frame: self.frame,
        });
    }
}
