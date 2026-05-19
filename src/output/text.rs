use crate::domain::category::Category;
use std::io::{self, Write};

pub fn write_categories<W: Write>(w: &mut W, cats: &[Category]) -> io::Result<()> {
    for c in cats {
        writeln!(w, "{} | {}", c.id, c.name)?;
    }
    Ok(())
}
