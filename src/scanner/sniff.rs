use std::io::{self, Read};
use std::path::Path;

const SNIFF_LEN: u64 = 8 * 1024;

pub fn is_binary(path: &Path) -> io::Result<bool> {
    let mut head = Vec::with_capacity(SNIFF_LEN as usize);
    std::fs::File::open(path)?
        .take(SNIFF_LEN)
        .read_to_end(&mut head)?;
    Ok(head.contains(&0))
}
