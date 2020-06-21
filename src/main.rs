use termion;
mod compression;
mod fat_bongo;
mod oled;
mod slim_bongo;

const IDLE_FRAMES: usize = 5;
const TAP_FRAMES: usize = 2;
const NUM_COLS: usize = 128;
const SLEEP_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);
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

fn display_bongo(get_frame: &dyn Fn(&Frame) -> Vec<u8>) {
    for frame in all_frames().iter().map(|frame| get_frame(frame)) {
        print!(
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            oled::render(&frame, NUM_COLS)
        );
        std::thread::sleep(SLEEP_INTERVAL);
    }
}

fn main() {
    display_bongo(&fat_bongo::get_frame);
    let fat_frames = all_frames()
        .iter()
        .map(|frame| fat_bongo::get_frame(frame))
        .collect::<Vec<_>>();

    let slim = compression::compress_frames(&fat_frames);
    println!("{}", slim);
    display_bongo(&slim_bongo::get_frame);
}
