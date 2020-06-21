use super::Frame;

const BASE_FRAME: [u8; 394] = [
    0, 2, 126, 126, 24, 60, 102, 66, 0, 1, 12, 28, 112, 112, 28, 12, 0, 1, 116, 116, 20, 20, 124,
    104, 0, 1, 124, 124, 0, 1, 112, 120, 44, 36, 124, 124, 0, 26, 128, 64, 64, 32, 32, 32, 32, 16,
    16, 16, 16, 16, 8, 8, 4, 4, 4, 8, 48, 64, 128, 0, 46, 128, 128, 128, 0, 4, 192, 96, 48, 24, 12,
    132, 198, 98, 35, 51, 17, 145, 113, 241, 113, 145, 17, 51, 35, 98, 198, 132, 12, 24, 48, 96,
    192, 0, 18, 24, 100, 130, 2, 2, 2, 2, 2, 1, 0, 4, 128, 128, 0, 9, 128, 0, 1, 48, 48, 0, 1, 192,
    193, 193, 194, 4, 8, 16, 32, 64, 128, 0, 3, 128, 128, 128, 128, 64, 64, 64, 64, 32, 32, 32, 32,
    16, 16, 16, 16, 8, 8, 8, 8, 8, 196, 4, 196, 4, 196, 2, 194, 2, 194, 1, 1, 1, 1, 0, 5, 252, 15,
    1, 0, 1, 248, 14, 31, 109, 140, 148, 148, 164, 166, 249, 224, 255, 224, 249, 166, 164, 148,
    148, 140, 109, 31, 14, 248, 0, 1, 1, 15, 252, 0, 15, 192, 56, 4, 3, 0, 7, 12, 12, 12, 13, 1, 0,
    1, 64, 160, 33, 34, 18, 17, 17, 17, 9, 8, 8, 8, 8, 4, 4, 8, 8, 16, 16, 16, 16, 16, 17, 15, 1,
    1, 0, 18, 170, 170, 255, 255, 195, 191, 127, 3, 127, 191, 195, 255, 255, 170, 170, 0, 6, 31,
    120, 192, 0, 1, 15, 56, 124, 219, 152, 20, 20, 18, 50, 207, 3, 255, 3, 207, 50, 18, 20, 20,
    152, 219, 124, 56, 15, 0, 1, 192, 120, 31, 16, 16, 16, 16, 8, 8, 8, 8, 8, 4, 4, 4, 4, 4, 2, 3,
    2, 2, 1, 1, 1, 1, 1, 1, 2, 2, 4, 4, 8, 8, 8, 8, 8, 7, 0, 42, 2, 130, 135, 31, 7, 159, 7, 28, 7,
    159, 7, 159, 7, 2, 130, 0, 4, 32, 16, 16, 16, 17, 11, 14, 12, 24, 16, 49, 35, 98, 102, 68, 68,
    71, 71, 71, 68, 68, 102, 98, 35, 49, 16, 24, 12, 6, 3, 1, 0, 78, 7, 8, 8, 23, 0, 1, 15, 1, 2,
    1, 15, 0, 1, 15, 2, 5, 8,
];

const IDLE0: [u8; 6] = [0, 255, 0, 255, 0, 126];
const IDLE1: [u8; 6] = [0, 255, 0, 255, 0, 126];
const IDLE2: [u8; 81] = [
    0, 59, 128, 128, 64, 64, 64, 64, 32, 32, 32, 32, 0, 2, 4, 2, 2, 0, 1, 24, 96, 128, 0, 99, 60,
    194, 1, 1, 0, 2, 4, 4, 2, 1, 0, 16, 96, 96, 0, 1, 129, 130, 130, 132, 8, 16, 32, 64, 128, 0,
    89, 128, 112, 25, 6, 0, 7, 24, 24, 24, 27, 3, 0, 3, 34, 36, 20, 18, 18, 18, 11, 0, 4, 5, 5, 9,
    9, 0, 255, 0, 43,
];
const IDLE3: [u8; 30] = [
    0, 51, 128, 128, 0, 16, 8, 4, 2, 1, 1, 2, 12, 0, 101, 30, 225, 0, 2, 1, 1, 0, 121, 128, 112,
    12, 0, 255, 0, 74,
];
const IDLE4: [u8; 26] = [
    0, 69, 8, 0, 1, 4, 2, 2, 2, 4, 56, 0, 100, 28, 226, 1, 1, 0, 123, 128, 112, 12, 0, 255, 0, 74,
];
const PREP: [u8; 74] = [
    0, 51, 128, 128, 0, 16, 8, 4, 2, 1, 1, 2, 12, 0, 101, 30, 225, 0, 2, 1, 1, 0, 2, 129, 128, 128,
    0, 19, 1, 225, 26, 6, 9, 49, 53, 1, 138, 124, 0, 87, 128, 112, 12, 0, 3, 24, 6, 5, 152, 153,
    132, 195, 124, 65, 65, 64, 0, 1, 32, 0, 13, 4, 4, 4, 4, 2, 2, 2, 1, 1, 0, 255, 0, 36,
];
const TAP0: [u8; 99] = [
    0, 51, 128, 128, 0, 16, 8, 4, 2, 1, 1, 2, 12, 0, 10, 248, 248, 248, 248, 0, 5, 128, 128, 0, 80,
    30, 225, 0, 2, 1, 1, 0, 2, 129, 128, 128, 0, 19, 1, 1, 2, 0, 4, 67, 135, 7, 1, 0, 1, 184, 188,
    190, 159, 95, 95, 79, 76, 0, 77, 128, 112, 12, 0, 3, 24, 6, 5, 152, 153, 132, 67, 124, 65, 65,
    64, 0, 1, 32, 0, 23, 61, 124, 252, 252, 252, 252, 252, 60, 12, 0, 61, 63, 0, 60, 1, 3, 3, 0,
    156,
];
const TAP1: [u8; 89] = [
    0, 51, 128, 128, 0, 16, 8, 4, 2, 1, 1, 2, 12, 0, 101, 30, 225, 0, 2, 1, 1, 0, 24, 1, 225, 26,
    6, 9, 49, 53, 1, 138, 124, 0, 87, 128, 112, 12, 0, 11, 1, 0, 17, 4, 4, 4, 4, 2, 2, 2, 1, 1, 0,
    88, 122, 122, 121, 121, 121, 121, 57, 49, 0, 7, 136, 136, 135, 128, 0, 113, 48, 120, 124, 254,
    255, 63, 7, 0, 4, 255, 255, 127, 127, 63, 62, 28, 24, 0, 52,
];

use crate::compression;
pub(super) fn get_frame(frame: &Frame) -> Vec<u8> {
    let compact = match *frame {
        Frame::Idle(0) => IDLE0.iter(),
        Frame::Idle(1) => IDLE1.iter(),
        Frame::Idle(2) => IDLE2.iter(),
        Frame::Idle(3) => IDLE3.iter(),
        Frame::Idle(4) => IDLE4.iter(),
        Frame::Prep => PREP.iter(),
        Frame::Tap(0) => TAP0.iter(),
        Frame::Tap(1) => TAP1.iter(),
        _ => panic!("Unexpected frame requested"),
    }.cloned().collect::<Vec<_>>();
    let original = compression::CompressedBytes::from_compressed(&BASE_FRAME).to_uncompressed();
    let diff = compression::CompressedBytes::from_compressed(&compact).to_uncompressed();
    compression::reconstruct_frame(&original, &diff)


}
