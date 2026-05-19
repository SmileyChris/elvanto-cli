use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "elvanto", version, about = "CLI for the Elvanto API", long_about = None)]
pub struct Cli {
    /// Print extra diagnostic information to stderr (credentials are redacted).
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Authentication utilities.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Worship song commands.
    Songs {
        #[command(subcommand)]
        command: SongsCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Verify the configured API key works.
    Check,
}

#[derive(Debug, Subcommand)]
pub enum SongsCommand {
    /// List song categories.
    Categories(JsonOnly),
    /// List active songs (all pages).
    List(SongsListArgs),
    /// Show a song by id.
    Show(SongsShowArgs),
    /// Print the chord chart for a song's default arrangement.
    Chart(SongsChartArgs),
    /// Print the lyrics for a song's default arrangement.
    Lyrics(SongsLyricsArgs),
}

#[derive(Debug, Args)]
pub struct JsonOnly {
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct SongsListArgs {
    /// Emit normalized JSON; includes non-active songs.
    #[arg(long)]
    pub json: bool,
    /// Include the album column in text output.
    #[arg(long)]
    pub album: bool,
    /// Include the CCLI number column in text output.
    #[arg(long)]
    pub ccli: bool,
}

#[derive(Debug, Args)]
pub struct SongsShowArgs {
    pub id: String,
    /// Emit normalized JSON of the full song object.
    #[arg(long)]
    pub json: bool,
    /// Expand all fields in text output (excluding lyrics/chord chart).
    #[arg(long)]
    pub full: bool,
    /// Include attachment data (only meaningful with --json).
    #[arg(long)]
    pub files: bool,
}

#[derive(Debug, Args)]
pub struct SongsChartArgs {
    pub id: String,
    /// Transpose to a named key (C, F#, Bb) or a relative offset (-2, +3).
    #[arg(long)]
    pub transpose: Option<String>,
    /// Use this arrangement instead of the default.
    #[arg(long)]
    pub arrangement: Option<String>,
}

#[derive(Debug, Args)]
pub struct SongsLyricsArgs {
    pub id: String,
    /// Use this arrangement instead of the default.
    #[arg(long)]
    pub arrangement: Option<String>,
}
