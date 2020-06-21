mod fat_bongo;
mod oled;

const IDLE_FRAMES: usize = 5;
const TAP_FRAMES: usize = 2;

enum Frame {
    Idle(usize),
    Prep,
    Tap(usize),
}

fn all_frames() -> Vec<Frame> {
    let mut all_frames = (0..IDLE_FRAMES)
        .map(|index| Frame::Idle(index))
        .collect::<Vec<Frame>>();
    all_frames.extend(std::iter::once(Frame::Prep));
    all_frames.extend((0..TAP_FRAMES).map(|index| Frame::Tap(index)));
    all_frames
}

fn main() {
    let num_cols = 128;

    for frame in all_frames().iter().map(|frame| fat_bongo::get_frame(frame)) {
        println!("{}", oled::render(frame, num_cols));
    }
}
