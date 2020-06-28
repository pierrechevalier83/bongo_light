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
    fn flattened_regions(&self) -> Vec<usize> {
        self.differing_regions
            .iter()
            .flat_map(|region| std::iter::once(region.begin).chain(std::iter::once(region.end)))
            .collect()
    }
}

pub(super) struct CompactAnimation {
    frame_len: usize,
    original: Diff,
    all_frames: Vec<Diff>,
}

impl CompactAnimation {
    pub fn construct_empty_frame(frame_len: usize) -> Vec<u8> {
        (0..frame_len).map(|_| 0).collect()
    }
    fn original_frame(&self) -> Vec<u8> {
        self.original
            .reconstruct_frame(&Self::construct_empty_frame(self.frame_len))
    }
    fn from_original(original: &[u8]) -> Self {
        // Store original as a diff to an empty screen to save a few bytes
        let frame_len = original.len();
        let original =
            Diff::from_original_and_altered(&Self::construct_empty_frame(frame_len), original);
        Self {
            frame_len,
            original,
            all_frames: Vec::new(),
        }
    }
    fn with_frame(mut self, other_frame: &[u8]) -> Self {
        let diff = Diff::from_original_and_altered(&self.original_frame(), other_frame);
        self.all_frames.push(diff);
        self
    }
    fn differing_regions_boundaries(&self) -> Vec<usize> {
        let mut index: usize = 0;
        std::iter::once({
            index += self.original.flattened_regions().len();
            index
        })
        .chain(self.all_frames.iter().map(|frame| {
            index += frame.flattened_regions().len();
            index
        }))
        .collect()
    }
    fn differing_regions(&self) -> Vec<usize> {
        self.original
            .flattened_regions()
            .iter()
            .cloned()
            .chain(
                self.all_frames
                    .iter()
                    .flat_map(|frame| frame.flattened_regions().into_iter()),
            )
            .collect()
    }
    fn differing_bytes_boundaries(&self) -> Vec<usize> {
        let mut index: usize = 0;
        std::iter::once({
            index += self.original.diff.len();
            index
        })
        .chain(self.all_frames.iter().map(|frame| {
            index += frame.diff.len();
            index
        }))
        .collect()
    }
    fn differing_bytes(&self) -> Vec<u8> {
        self.original
            .diff
            .iter()
            .cloned()
            .chain(
                self.all_frames
                    .iter()
                    .flat_map(|frame| frame.diff.iter().cloned()),
            )
            .collect()
    }
}

fn as_c_array_string<T: std::fmt::Debug>(v: &[T]) -> String {
    format!("{:?}", v).replace("[", "{").replace("]", "}")
}

fn fmt_as_rust_array<T: std::fmt::Debug>(name: &str, rust_type: &str, array: &[T]) -> String {
    format!(
        "const {}: [{}; {}] = {:?};",
        name,
        rust_type,
        array.len(),
        array,
    )
}

fn fmt_as_c_array<T: std::fmt::Debug>(name: &str, c_type: &str, array: &[T]) -> String {
    format!(
        "static const {} PROGMEM {}[{}] = {};",
        c_type,
        name,
        array.len(),
        as_c_array_string(array),
    )
}

impl std::fmt::Display for CompactAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#### Rust ####")?;
        writeln!(f, "```")?;
        writeln!(f, "const FRAME_SIZE: usize = {};", self.frame_len)?;
        writeln!(
            f,
            "{}",
            fmt_as_rust_array(
                "DIFF_REGIONS_BOUNDARIES",
                "usize",
                &self.differing_regions_boundaries()
            )
        )?;
        writeln!(
            f,
            "{}",
            fmt_as_rust_array("DIFF_REGIONS", "usize", &self.differing_regions())
        )?;
        writeln!(
            f,
            "{}",
            fmt_as_rust_array(
                "DIFF_BYTES_BOUNDARIES",
                "usize",
                &self.differing_bytes_boundaries()
            )
        )?;
        writeln!(
            f,
            "{}",
            fmt_as_rust_array("DIFF_BYTES", "u8", &self.differing_bytes())
        )?;
        writeln!(f, "```")?;
        writeln!(f, "#### C ####")?;
        writeln!(f, "```")?;
        writeln!(f, "#define FRAME_SIZE {}", self.frame_len)?;
        writeln!(
            f,
            "{}",
            fmt_as_c_array(
                "diff_regions_boundaries",
                "uint16_t",
                &self.differing_regions_boundaries()
            )
        )?;
        writeln!(
            f,
            "{}",
            fmt_as_c_array("diff_regions", "uint16_t", &self.differing_regions())
        )?;
        writeln!(
            f,
            "{}",
            fmt_as_c_array(
                "diff_bytes_boundaries",
                "uint16_t",
                &self.differing_bytes_boundaries()
            )
        )?;
        writeln!(
            f,
            "{}",
            fmt_as_c_array("diff_bytes", "char", &self.differing_bytes())
        )?;
        writeln!(f, "```")?;
        let size = self.differing_bytes().len()
            + 2 * (self.differing_bytes_boundaries().len()
                + self.differing_regions().len()
                + self.differing_regions_boundaries().len());
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
