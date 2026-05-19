use crate::domain::category::Category;
use crate::domain::song::SongSummary;
use std::io::{self, Write};

pub fn write_categories<W: Write>(w: &mut W, cats: &[Category]) -> io::Result<()> {
    for c in cats {
        writeln!(w, "{} | {}", c.id, c.name)?;
    }
    Ok(())
}

pub fn write_songs<W: Write>(
    w: &mut W,
    songs: &[SongSummary],
    show_album: bool,
    show_ccli: bool,
) -> io::Result<()> {
    for s in songs {
        write!(w, "{} | {} | {}", s.id, s.title, s.artist)?;
        if show_album {
            write!(w, " | {}", s.album)?;
        }
        if show_ccli {
            write!(w, " | {}", s.ccli_number)?;
        }
        writeln!(w)?;
    }
    Ok(())
}
