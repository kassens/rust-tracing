use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::frame::{get_frames_clone, register_frame};
use crate::speedscope::{
    Event, EventedProfile, File, FileShared, Frame, FrameIndex, ProfileTypeEvented,
    SpeedscopeSchemaUrl, Value, ValueUnit,
};

use std::fmt::Display;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

pub struct Profiler {
    start_value: Value,
    profiles: Arc<Mutex<Vec<EventedProfile>>>,
}
impl Profiler {
    pub fn new() -> Self {
        Self {
            start_value: elapsed(),
            profiles: Default::default(),
        }
    }
    pub fn thread(&self, name: impl Into<String>) -> Thread {
        Thread::new(name.into(), Arc::clone(&self.profiles))
    }
    pub fn complete(self) -> Option<File> {
        let end_value = elapsed();
        let mut threads = Arc::try_unwrap(self.profiles).ok()?.into_inner();
        threads.sort_by_key(|t| t.start_value);

        // Set uniform start/end values to help understand relative time.
        // Otherwise, everything is visualized to start at the beginning.
        for t in &mut threads {
            t.start_value = self.start_value;
            t.end_value = end_value;
        }
        Some(File {
            schema: SpeedscopeSchemaUrl,
            shared: FileShared {
                frames: get_frames_clone(),
            },
            profiles: threads,
            name: None,
            active_profile_index: None,
            exporter: None,
        })
    }
}

pub struct Thread {
    name: String,
    profiles: Arc<Mutex<Vec<EventedProfile>>>,
    start_value: Value,
    events: Vec<Event>,
    needs_complete: NeedsComplete,
}
impl Thread {
    fn new(name: String, profiles: Arc<Mutex<Vec<EventedProfile>>>) -> Self {
        let start_value = elapsed();
        Self {
            name: name.clone(),
            profiles,
            start_value,
            events: vec![Event::OpenFrame {
                at: start_value,
                frame: register_frame(Frame {
                    name,
                    file: None,
                    line: None,
                    col: None,
                }),
            }],
            needs_complete: NeedsComplete::new("Thread"),
        }
    }
    pub fn span<'a>(&'a mut self, frame: FrameIndex) -> Span<'a> {
        Span::new(self, frame)
    }
    pub fn complete(mut self) {
        self.needs_complete.complete();
        let end_value = elapsed();
        self.events.push(Event::CloseFrame {
            at: end_value,
            frame: self.events[0].frame(),
        });
        self.profiles.lock().push(EventedProfile {
            type_: ProfileTypeEvented,
            name: self.name,
            unit: ValueUnit::Microseconds,
            start_value: self.start_value,
            end_value,
            events: self.events,
        });
    }
}

fn elapsed() -> Value {
    static START: Lazy<Instant> = Lazy::new(Instant::now);
    Value::new(START.elapsed().as_micros() as u64)
}

pub struct Span<'a> {
    thread: &'a mut Thread,
    frame: FrameIndex,
}
impl<'a> Span<'a> {
    fn new(thread: &'a mut Thread, frame: FrameIndex) -> Self {
        thread.events.push(Event::OpenFrame {
            at: elapsed(),
            frame,
        });
        Self { thread, frame }
    }

    #[must_use]
    pub fn span<'b>(&'b mut self, frame: FrameIndex) -> Span<'b> {
        Span::new(&mut self.thread, frame)
    }

    #[must_use]
    pub fn fork(&self, name: &impl Display) -> Thread {
        Thread::new(
            format!("{}/{}", &self.thread.name, name),
            Arc::clone(&self.thread.profiles),
        )
    }
}
impl<'a> Drop for Span<'a> {
    fn drop(&mut self) {
        self.thread.events.push(Event::CloseFrame {
            at: elapsed(),
            frame: self.frame,
        });
    }
}

struct NeedsComplete {
    name: &'static str,
    completed: bool,
}

impl NeedsComplete {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            completed: false,
        }
    }
    fn complete(&mut self) {
        assert!(!self.completed, "can only complete() once");
        self.completed = true;
    }
}
impl Drop for NeedsComplete {
    fn drop(&mut self) {
        if !self.completed && !thread::panicking() {
            println!(
                "Expected {}::complete() to be called before being dropped.",
                self.name
            )
        }
    }
}
