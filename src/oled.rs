use itertools::*;

/// Emulate an oled display in the command line:
/// Take a slice of bytes where each byte represents 8 rows of the display
pub fn render(data: &[u8], num_cols: usize) -> String {
    data.chunks(num_cols)
        .map(move |bytes| {
            (0..8)
                .rev()
                .map(|bit_index| {
                    let pixels = bytes.iter().map(|byte| {
                        let bits = format!("{:08b}", byte);
                        match bits.chars().nth(bit_index) {
                            Some('1') => "██",
                            Some('0') => "  ",
                            _ => panic!("Bit can only be 0 or 1"),
                        }
                    });
                    pixels.collect::<String>()
                })
                .intersperse("\n".to_string())
                .collect::<String>()
        })
        .map(|s| s + "\n")
        .collect()
}
