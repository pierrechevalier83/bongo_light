use itertools::Itertools;

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
            compressed: compressed.iter().cloned().collect(),
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

struct Range {
    begin: usize,
    end: usize,
}

impl Range {
    fn contains(&self, index: usize) -> bool {
        index >= self.begin && index < self.end
    }
}

impl std::fmt::Debug for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.begin, self.end)
    }
}

pub struct CompressedDiff {
    differing_regions: Vec<Range>,
    diff: CompressedBytes,
}

impl CompressedDiff {
    pub fn from_original_and_altered(original: &[u8], altered: &[u8]) -> Self {
        let mut differing_regions = Vec::<Range>::new();

        let diff = original
            .iter()
            .enumerate()
            .zip(altered.iter())
            .filter_map(|((index, o), m)| {
                if *o == *m {
                    None
                } else {
                    if let Some(mut last_range) = differing_regions.last_mut() {
                        if last_range.end == index {
                            last_range.end += 1;
                        } else {
                            differing_regions.push(Range {
                                begin: index,
                                end: index + 1,
                            })
                        }
                    } else {
                        differing_regions.push(Range {
                            begin: index,
                            end: index + 1,
                        })
                    }
                    Some(*m)
                }
            })
            .collect::<Vec<u8>>();

        Self {
            differing_regions,
            diff: CompressedBytes::from_uncompressed(&diff),
        }
    }
    pub fn from_regions_and_diff(regions: &[usize], diff: &[u8]) -> Self {
        let differing_regions = regions
            .into_iter()
            .chunks(2)
            .into_iter()
            .map(move |chunk| {
                let mut it = chunk.into_iter();
                Range {
                    begin: *it.next().unwrap(),
                    end: *it.next().unwrap(),
                }
            })
            .collect();
        let diff = CompressedBytes::from_compressed(diff);
        Self {
            differing_regions,
            diff,
        }
    }
    pub fn reconstruct_frame(&self, original: &CompressedBytes) -> Vec<u8> {
        let mut diff_index = 0;
        let diff = self.diff.to_uncompressed();
        original
            .to_uncompressed()
            .iter()
            .enumerate()
            .map(|(index, o)| {
                if self.index_differs(index) {
                    let d = diff[diff_index];
                    diff_index += 1;
                    d
                } else {
                    *o
                }
            })
            .collect()
    }
    fn index_differs(&self, index: usize) -> bool {
        for region in self.differing_regions.iter() {
            if region.contains(index) {
                return true;
            }
        }
        false
    }
}

pub(super) struct CompressedAnimation {
    original: CompressedBytes,
    all_frames: Vec<CompressedDiff>,
}

impl CompressedAnimation {
    fn from_original(original_frame: &[u8]) -> Self {
        Self {
            original: CompressedBytes::from_uncompressed(original_frame),
            all_frames: Vec::new(),
        }
    }
    fn with_frame(mut self, other_frame: &[u8]) -> Self {
        let original = self.original.to_uncompressed();
        let diff = CompressedDiff::from_original_and_altered(&original, other_frame);
        self.all_frames.push(diff);
        self
    }
}

impl std::fmt::Display for CompressedAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "const BASE_FRAME: [u8; {}] = {:?};",
            self.original.compressed.len(),
            self.original.compressed
        )?;
        writeln!(f, "")?;
        for (index, frame) in self.all_frames.iter().enumerate() {
            writeln!(
                f,
                "const REGIONS_{}: [usize; {}] = {:?};",
                index,
                frame.differing_regions.len() * 2,
                frame.differing_regions
            )?;
            writeln!(
                f,
                "const DIFF_{}: [u8; {}] = {:?};",
                index,
                frame.diff.compressed.len(),
                frame.diff.compressed
            )?;
        }
        let mut size: usize = self.original.compressed.len();
        size += self
            .all_frames
            .iter()
            .map(|frame| frame.diff.compressed.len() + 4 * frame.differing_regions.len())
            .sum::<usize>();
        writeln!(f, "Total size in bytes: {}", size)
    }
}
pub(super) fn compress_frames(all_frames: &[Vec<u8>]) -> CompressedAnimation {
    let mut slim = CompressedAnimation::from_original(&all_frames[0]);
    for frame in all_frames {
        slim = slim.with_frame(frame)
    }
    slim
}
