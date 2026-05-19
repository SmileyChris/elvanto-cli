use serde::Serialize;
use std::io::{self, Write};

pub fn write_pretty<W: Write, T: Serialize>(w: &mut W, value: &T) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *w, value)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(w)
}
