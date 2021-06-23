use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::speedscope::{Frame, FrameIndex};

#[macro_export]
macro_rules! frame {
    ($name:literal) => {{
        use once_cell::sync::Lazy;
        use $crate::speedscope::{Frame, FrameIndex};
        static FRAME: Lazy<FrameIndex> = Lazy::new(|| {
            $crate::frame::register_frame(Frame {
                name: String::from($name),
                file: Some(file!()),
                line: Some(line!()),
                col: Some(column!()),
            })
        });
        *FRAME
    }};
}

static FRAMES: Lazy<Mutex<Vec<Frame>>> = Lazy::new(Default::default);

pub fn register_frame(frame: Frame) -> FrameIndex {
    let mut frames = FRAMES.lock();
    let id = frames.len() as u32;
    frames.push(frame);
    FrameIndex::new(id)
}

pub fn get_frames_clone() -> Vec<Frame> {
    FRAMES.lock().clone()
}
