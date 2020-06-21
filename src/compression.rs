/// A naive compression algorithm that should be good enough for the case where
/// we are compressing diffs between similar frames:
/// keep the counts of zeroes instead of all of the zeroes.
/// so [0, 0, 0, 1, 1, 0, 1]
/// would become [0, 3, 1, 1, 0, 1, 1]
/// If a slice contains more than 255 zeroes in a row, break down the count like so:
/// [0, 255, 0, 255, 0, 8]
pub struct CompressedBytes {
    compressed: Vec<u8>,
}

impl CompressedBytes {
    fn from_uncompressed(bytes: &[u8]) -> Self {
        let mut compressed = Vec::new();
        enum State {
            CountingZeroes(u8),
            StoringNonZeroes,
        };
        let mut state = State::StoringNonZeroes;
        for byte in bytes {
            state = match state {
                State::StoringNonZeroes => {
                    if *byte == 0 {
                        State::CountingZeroes(1)
                    } else {
                        compressed.push(*byte);
                        State::StoringNonZeroes
                    }
                }
                State::CountingZeroes(count) => {
                    if *byte == 0 {
                        if count == u8::max_value() {
                            compressed.push(0);
                            compressed.push(count);
                            State::CountingZeroes(1)
                        } else {
                            // Do not add this zero. This is where compression happens
                            State::CountingZeroes(count + 1)
                        }
                    } else {
                        compressed.push(0);
                        compressed.push(count);
                        compressed.push(*byte);
                        State::StoringNonZeroes
                    }
                }
            }
        }
        match state {
            State::CountingZeroes(count) => {
                compressed.push(0);
                compressed.push(count);
            }
            _ => (),
        }
        Self { compressed }
    }
    pub fn from_compressed(compressed: &[u8]) -> Self {
        Self {
            compressed: compressed.iter().cloned().collect()
        }
    }
    pub fn to_uncompressed(&self) -> Vec<u8> {
        let mut uncompressed = Vec::new();
        enum State {
            ExpandingZeroes,
            ExpandingNonZeroes,
        }
        let mut state = State::ExpandingNonZeroes;
        for byte in self.compressed.iter() {
            state = match state {
                State::ExpandingZeroes => {
                    if *byte == 0 {
                        panic!("Compressed data may not contain two consecutive zeroes");
                    } else {
                        let num_zeroes = *byte;
                        uncompressed.extend((0..num_zeroes).map(|_| 0));
                        State::ExpandingNonZeroes
                    }
                }
                State::ExpandingNonZeroes => {
                    if *byte == 0 {
                        State::ExpandingZeroes
                    } else {
                        uncompressed.push(*byte);
                        State::ExpandingNonZeroes
                    }
                }
            }
        }
        uncompressed
    }
}

fn calculate_diff_between_frames(original: &[u8], modded: &[u8]) -> Vec<u8> {
    original
        .iter()
        .zip(modded.iter())
        .map(|(o, m)| if *o == *m { 0 } else { *m })
        .collect()
}

pub fn reconstruct_frame(original: &[u8], diff: &[u8]) -> Vec<u8> {
    original
        .iter()
        .zip(diff.iter())
        .map(|(o, d)| if *d == 0 { *o } else { *d })
        .collect()
}

pub(super) struct SlimBongo {
    original: CompressedBytes,
    other_frames_as_diffs: Vec<CompressedBytes>,
}

impl SlimBongo {
    fn from_original(original_frame: &[u8]) -> Self {
        Self {
            original: CompressedBytes::from_uncompressed(original_frame),
            other_frames_as_diffs: Vec::new(),
        }
    }
    fn with_frame(mut self, other_frame: &[u8]) -> Self {
        let original = self.original.to_uncompressed();
        let diff = calculate_diff_between_frames(&original, other_frame);
        self.other_frames_as_diffs
            .push(CompressedBytes::from_uncompressed(&diff));
        self
    }
}

impl std::fmt::Display for SlimBongo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "original:\n{:?}", self.original.compressed)?;
        writeln!(f, "all frames:")?;
        for frame in &self.other_frames_as_diffs {
            writeln!(f, "\t{:?}", frame.compressed)?;
        }
        let size = self.original.compressed.len()
            + self
                .other_frames_as_diffs
                .iter()
                .map(|frame| frame.compressed.len())
                .count();
        writeln!(f, "Total size in bytes: {}", size)
    }
}
pub(super) fn compress_frames(all_frames: &[Vec<u8>]) -> SlimBongo {
    let mut slim = SlimBongo::from_original(&all_frames[0]);
    for frame in all_frames {
        slim = slim.with_frame(frame)
    }
    slim
}
