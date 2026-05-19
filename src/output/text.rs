use crate::domain::category::Category;
use crate::domain::song::{SongDetail, SongSummary};
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

pub fn write_song_curated<W: Write>(w: &mut W, song: &SongDetail) -> io::Result<()> {
    writeln!(w, "Title:       {}", song.title)?;
    writeln!(w, "Artist:      {}", song.artist)?;
    writeln!(w, "CCLI number: {}", song.ccli_number)?;
    writeln!(w, "Status:      {}", song.status)?;

    let first_line = song
        .arrangements
        .iter()
        .find_map(|a| a.lyrics.as_deref())
        .and_then(|l| l.lines().find(|l| !l.trim().is_empty()))
        .unwrap_or("");
    writeln!(w, "First line:  {first_line}")?;

    writeln!(w, "Arrangements:")?;
    for arr in &song.arrangements {
        let keys: Vec<String> = arr
            .keys
            .iter()
            .map(|k| match &k.ending {
                Some(e) => format!("{}\u{2192}{}", k.starting, e),
                None => k.starting.clone(),
            })
            .collect();
        let keys_str = if keys.is_empty() { "\u{2014}".into() } else { keys.join(", ") };
        writeln!(w, "  - {} [{}]", arr.name, keys_str)?;
    }
    Ok(())
}

pub fn write_song_full<W: Write>(w: &mut W, song: &SongDetail) -> io::Result<()> {
    writeln!(w, "Title:           {}", song.title)?;
    writeln!(w, "Artist:          {}", song.artist)?;
    writeln!(w, "Album:           {}", song.album)?;
    writeln!(w, "CCLI number:     {}", song.ccli_number)?;
    writeln!(w, "Status:          {}", song.status)?;
    if let Some(v) = &song.sequence {
        writeln!(w, "Sequence:        {v}")?;
    }
    if let Some(v) = &song.bpm {
        writeln!(w, "BPM:             {v}")?;
    }
    if let Some(v) = &song.duration {
        writeln!(w, "Duration:        {v}")?;
    }
    writeln!(w, "Learn:           {}", song.learn)?;
    writeln!(w, "Allow downloads: {}", song.allow_downloads)?;
    if !song.categories.is_empty() {
        let names: Vec<&str> = song.categories.iter().map(|c| c.name.as_str()).collect();
        writeln!(w, "Categories:      {}", names.join(", "))?;
    }
    if !song.locations.is_empty() {
        let names: Vec<&str> = song.locations.iter().map(|c| c.name.as_str()).collect();
        writeln!(w, "Locations:       {}", names.join(", "))?;
    }
    if let Some(n) = &song.notes {
        writeln!(w, "Notes:           {n}")?;
    }
    writeln!(w, "Arrangements:")?;
    for arr in &song.arrangements {
        let keys: Vec<String> = arr.keys.iter().map(|k| k.starting.clone()).collect();
        writeln!(w, "  - {} [{}]", arr.name, keys.join(", "))?;
    }
    Ok(())
}
