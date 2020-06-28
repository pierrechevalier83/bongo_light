use itertools::Itertools;

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

pub struct Diff {
    differing_regions: Vec<Range>,
    diff: Vec<u8>,
}

impl Diff {
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
            diff,
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
        Self {
            differing_regions,
            diff: diff.iter().cloned().collect(),
        }
    }
    pub fn reconstruct_frame(&self, original: &[u8]) -> Vec<u8> {
        let mut diff_index = 0;
        original
            .iter()
            .enumerate()
            .map(|(index, o)| {
                if self.index_differs(index) {
                    let d = self.diff[diff_index];
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

pub(super) struct CompactAnimation {
    original: Vec<u8>,
    all_frames: Vec<Diff>,
}

impl CompactAnimation {
    fn from_original(original: &[u8]) -> Self {
        Self {
            original: original.iter().cloned().collect(),
            all_frames: Vec::new(),
        }
    }
    fn with_frame(mut self, other_frame: &[u8]) -> Self {
        let diff = Diff::from_original_and_altered(&self.original, other_frame);
        self.all_frames.push(diff);
        self
    }
}

fn as_c_array_string<T: std::fmt::Debug>(v: &Vec<T>) -> String {
    format!("{:?}", v).replace("[", "{").replace("]", "}")
}

impl std::fmt::Display for CompactAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#### Rust ####")?;
        writeln!(f, "```")?;
        writeln!(
            f,
            "const BASE_FRAME: [u8; {}] = {:?};",
            self.original.len(),
            self.original
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
                frame.diff.len(),
                frame.diff
            )?;
        }
        writeln!(f, "```")?;
        writeln!(f, "#### C ####")?;
        writeln!(f, "```")?;
        writeln!(
            f,
            "static const char PROGMEM base_frame[{}] = {};",
            self.original.len(),
            as_c_array_string(&self.original)
        )?;
        writeln!(f, "")?;
        for (index, frame) in self.all_frames.iter().enumerate() {
            writeln!(
                f,
                "#define REGIONS_LEN_{} {}",
                index,
                frame.differing_regions.len() * 2,
            )?;
            writeln!(
                f,
                "static const uint16_t PROGMEM regions_{}[REGIONS_LEN_{}] = {};",
                index,
                index,
                as_c_array_string(&frame.differing_regions)
            )?;
            writeln!(f, "#define DIFF_LEN_{} {}", index, frame.diff.len(),)?;
            writeln!(
                f,
                "static const char PROGMEM diff_{}[DIFF_LEN_{}] = {};",
                index, index, as_c_array_string(&frame.diff)
            )?;
        }
        writeln!(f, "```")?;
        let mut size: usize = self.original.len();
        size += self
            .all_frames
            .iter()
            .map(|frame| frame.diff.len() + 4 * frame.differing_regions.len())
            .sum::<usize>();
        writeln!(f, "Total size in bytes: {}", size)
    }
}
pub(super) fn compress_frames(all_frames: &[Vec<u8>]) -> CompactAnimation {
    let mut slim = CompactAnimation::from_original(&all_frames[0]);
    for frame in all_frames {
        slim = slim.with_frame(frame)
    }
    slim
}
