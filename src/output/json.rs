use serde::Serialize;
use std::io::{self, Write};

pub fn write_pretty<W: Write, T: Serialize>(w: &mut W, value: &T) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *w, value).map_err(io::Error::other)?;
    writeln!(w)
}
