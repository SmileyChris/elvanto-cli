use crate::cli::IdMode;
use crate::domain::category::{self, Category};
use crate::domain::service::Service;
use crate::domain::song::{SongDetail, SongSummary};
use std::io::{self, Write};

/// Render the id column for `mode`, returning `None` when hidden so callers
/// can drop the column entirely (rather than emit a blank `| ` cell).
fn id_cell(full_id: &str, mode: IdMode) -> Option<&str> {
    match mode {
        IdMode::Hidden => None,
        IdMode::Long => Some(full_id),
        IdMode::Short => Some(category::short_id(full_id)),
    }
}

pub fn write_categories<W: Write>(w: &mut W, cats: &[Category], mode: IdMode) -> io::Result<()> {
    for c in cats {
        match id_cell(&c.id, mode) {
            Some(id) => writeln!(w, "{} | {}", id, c.name)?,
            None => writeln!(w, "{}", c.name)?,
        }
    }
    Ok(())
}

pub fn write_songs<W: Write>(
    w: &mut W,
    songs: &[SongSummary],
    show_album: bool,
    show_ccli: bool,
    mode: IdMode,
    show_last_used: bool,
) -> io::Result<()> {
    for s in songs {
        if let Some(id) = id_cell(&s.id, mode) {
            write!(w, "{} | ", id)?;
        }
        write!(w, "{} | {}", s.title, s.artist)?;
        if show_album {
            write!(w, " | {}", s.album)?;
        }
        if show_ccli {
            write!(w, " | {}", s.ccli_number)?;
        }
        if show_last_used {
            write!(w, " | {}", s.last_used.as_deref().unwrap_or("-"))?;
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
        let keys_str = if keys.is_empty() {
            "\u{2014}".into()
        } else {
            keys.join(", ")
        };
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
        let keys: Vec<String> = arr
            .keys
            .iter()
            .map(|k| match &k.ending {
                Some(e) => format!("{}\u{2192}{}", k.starting, e),
                None => k.starting.clone(),
            })
            .collect();
        let keys_str = if keys.is_empty() {
            "\u{2014}".into()
        } else {
            keys.join(", ")
        };
        writeln!(w, "  - {} [{}]", arr.name, keys_str)?;
    }
    Ok(())
}

pub fn write_services<W: Write>(w: &mut W, services: &[Service], mode: IdMode) -> io::Result<()> {
    for s in services {
        if let Some(id) = id_cell(&s.id, mode) {
            write!(w, "{} | ", id)?;
        }
        let location = s.location.as_deref().unwrap_or("-");
        writeln!(
            w,
            "{} | {} | {} | {} | {}",
            s.date_short(),
            s.name,
            s.service_type,
            location,
            s.status,
        )?;
    }
    Ok(())
}

use crate::domain::person::{DepartmentRow, Person};
use crate::domain::service::VolunteerRow;

pub fn write_people<W: Write>(w: &mut W, people: &[Person], mode: IdMode) -> io::Result<()> {
    for p in people {
        if let Some(id) = id_cell(&p.id, mode) {
            write!(w, "{} | ", id)?;
        }
        let email = p.email.as_deref().unwrap_or("-");
        writeln!(w, "{} | {}", p.name, email)?;
    }
    Ok(())
}

pub fn write_org_tree<W: Write>(w: &mut W, rows: &[DepartmentRow], mode: IdMode) -> io::Result<()> {
    for r in rows {
        if let Some(id) = id_cell(&r.id, mode) {
            write!(w, "{} | ", id)?;
        }
        // Indent by depth so the tree is visually obvious in text mode.
        // `kind` and `parent` are dropped from text output (depth + line order
        // already convey the same info); JSON still carries them.
        let indent = match r.kind.as_str() {
            "department" => "",
            "sub_department" => "  ",
            "position" => "    ",
            _ => "",
        };
        writeln!(w, "{}{}", indent, r.name)?;
    }
    Ok(())
}

pub fn write_service_people<W: Write>(
    w: &mut W,
    rows: &[VolunteerRow],
    show_email: bool,
    mode: IdMode,
) -> io::Result<()> {
    for r in rows {
        let dept = if r.sub_department.is_empty() {
            r.department.as_str()
        } else {
            r.sub_department.as_str()
        };
        let name = r.name.as_deref().unwrap_or("(unfilled)");
        let status = r.status.as_deref().unwrap_or("-");
        if let Some(person_id) = r.person_id.as_deref() {
            if let Some(id) = id_cell(person_id, mode) {
                write!(w, "{} | ", id)?;
            }
        } else if mode != IdMode::Hidden {
            // Unfilled row: keep the column aligned by emitting a `-` placeholder.
            write!(w, "- | ")?;
        }
        write!(w, "{} | {} | {} | {}", dept, r.position, name, status)?;
        if show_email {
            write!(w, " | {}", r.email.as_deref().unwrap_or("-"))?;
        }
        writeln!(w)?;
    }
    Ok(())
}
