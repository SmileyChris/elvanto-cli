use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "elvanto", version, about = "CLI for the Elvanto API", long_about = None)]
pub struct Cli {
    /// Print extra diagnostic information to stderr (credentials are redacted).
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Ignore ELVANTO_<SUBCOMMAND> env vars that auto-inject default flags.
    #[arg(long, global = true)]
    pub no_env: bool,

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
    /// People (directory) commands.
    People {
        #[command(subcommand)]
        command: PeopleCommand,
    },
    /// Service (calendar event) commands.
    Services {
        #[command(subcommand)]
        command: ServicesCommand,
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
    Categories(SongsCategoriesArgs),
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
pub struct SongsCategoriesArgs {
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
    /// Show full category UUIDs in text output.
    #[arg(long)]
    pub full_id: bool,
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
    /// Keep songs assigned to this category id; repeat for OR matching.
    #[arg(long = "category-id", value_name = "ID")]
    pub category_ids: Vec<String>,
    /// Show full song UUIDs in text output.
    #[arg(long)]
    pub full_id: bool,
    /// Include the most recent service date in text output.
    #[arg(long)]
    pub last_used: bool,
    /// Keep songs used in a service within this duration (e.g. 6m, 2w).
    #[arg(long, value_name = "DURATION")]
    pub used_within: Option<String>,
    /// Exclude songs used in a service within this duration (e.g. 2w, 14d).
    #[arg(long = "not-used-within", value_name = "DURATION")]
    pub not_used_within: Option<String>,
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

#[derive(Debug, Subcommand)]
pub enum ServicesCommand {
    /// List services in a date range (defaults to the last 6 months).
    List(ServicesListArgs),
    /// Show people assigned to a service (volunteers, by position).
    People(ServicesPeopleArgs),
}

#[derive(Debug, Subcommand)]
pub enum PeopleCommand {
    /// List active people (id, name, email). Optionally filter by department.
    List(PeopleListArgs),
    /// List unique departments and sub-departments (flat).
    Departments(PeopleDepartmentsArgs),
}

#[derive(Debug, Args)]
pub struct PeopleListArgs {
    /// Keep people whose department or sub-department matches (case-insensitive); repeat to OR-match.
    #[arg(long, value_name = "NAME")]
    pub department: Vec<String>,
    /// Print full UUIDs in text output (default uses short ids).
    #[arg(long)]
    pub full_id: bool,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct PeopleDepartmentsArgs {
    /// Print full UUIDs in text output (default uses short ids).
    #[arg(long)]
    pub full_id: bool,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ServicesPeopleArgs {
    pub id: String,
    /// Hide unfilled positions (default: show every position).
    #[arg(long)]
    pub hide_unfilled: bool,
    /// Keep rows where the department or sub-department matches (case-insensitive); repeat to OR-match.
    #[arg(long, value_name = "NAME")]
    pub department: Vec<String>,
    /// Include each person's primary email in the output.
    #[arg(long)]
    pub email: bool,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ServicesListArgs {
    /// Inclusive start date (YYYY-MM-DD). Defaults to 6 months before --to.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub from: Option<String>,
    /// Inclusive end date (YYYY-MM-DD). Defaults to today (local time).
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub to: Option<String>,
    /// Print full UUIDs in text output (default uses short ids).
    #[arg(long)]
    pub full_id: bool,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}
