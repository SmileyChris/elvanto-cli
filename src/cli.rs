use clap::{Args, Parser, Subcommand, ValueEnum};

/// How the id column is rendered in text-mode list output.
/// `short` (default) prints the first dash-separated UUID block; `long` prints
/// the full UUID; `hidden` omits the id column entirely. JSON output ignores
/// this and always emits the full UUID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum IdMode {
    #[default]
    Short,
    Long,
    Hidden,
}

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
    /// Store an API key in the OS keyring.
    Login(AuthLoginArgs),
    /// Remove the API key from the OS keyring.
    Clear,
    /// Show key source (env / keyring / none) and verify against Elvanto.
    Status,
}

#[derive(Debug, Clone, Args)]
pub struct AuthLoginArgs {
    /// Read the API key from stdin (one line) instead of prompting.
    #[arg(long)]
    pub stdin: bool,
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
    /// Id rendering: short (default), long, or hidden.
    #[arg(long = "id", value_enum, default_value_t = IdMode::Short, value_name = "MODE")]
    pub id_mode: IdMode,
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
    /// Keep songs assigned to this category. Accepts a full UUID, short
    /// first-block, or category name (case-insensitive). Repeat for OR
    /// matching. Use `elvanto songs categories` to browse.
    #[arg(long = "category", value_name = "ID|NAME")]
    pub category_ids: Vec<String>,
    /// Id rendering: short (default), long, or hidden.
    #[arg(long = "id", value_enum, default_value_t = IdMode::Short, value_name = "MODE")]
    pub id_mode: IdMode,
    /// Include the most recent service date in text output.
    #[arg(long)]
    pub last_used: bool,
    /// Include number of times sung in text output.
    #[arg(long)]
    pub count: bool,
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
    /// Use this arrangement id (full UUID or short first-block) instead of
    /// the default. Use `elvanto songs show <song>` to look up ids.
    #[arg(long, value_name = "ID")]
    pub arrangement: Option<String>,
}

#[derive(Debug, Args)]
pub struct SongsLyricsArgs {
    pub id: String,
    /// Use this arrangement id (full UUID or short first-block) instead of
    /// the default. Use `elvanto songs show <song>` to look up ids.
    #[arg(long, value_name = "ID")]
    pub arrangement: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ServicesCommand {
    /// List services in a date range (defaults to the last 6 months).
    List(ServicesListArgs),
    /// Show people assigned to a service (volunteers, by position).
    People(ServicesPeopleArgs),
    /// Analyse song usage across services.
    SongUsage(ServicesSongUsageArgs),
}

#[derive(Debug, Subcommand)]
pub enum PeopleCommand {
    /// List active people (id, name, email). Optionally filter by org tree.
    List(PeopleListArgs),
    /// Show the organisational tree (departments, sub-departments, positions).
    Org(PeopleOrgArgs),
}

#[derive(Debug, Args)]
pub struct PeopleListArgs {
    /// Keep people whose department, sub-department, OR position matches.
    /// Accepts a full UUID, short first-block, name (e.g. `Vocals`), or
    /// path (e.g. `Music Team/Vocals`). A parent match includes the whole
    /// subtree. Repeat for OR; use `elvanto people departments` to browse.
    #[arg(long = "in", value_name = "ID|NAME|PATH")]
    pub department: Vec<String>,
    /// Id rendering: short (default), long, or hidden.
    #[arg(long = "id", value_enum, default_value_t = IdMode::Short, value_name = "MODE")]
    pub id_mode: IdMode,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct PeopleOrgArgs {
    /// Id rendering: short (default), long, or hidden.
    #[arg(long = "id", value_enum, default_value_t = IdMode::Short, value_name = "MODE")]
    pub id_mode: IdMode,
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
    /// Keep rows whose department, sub-department, OR position matches.
    /// Accepts a full UUID, short first-block, name, or path
    /// (e.g. `Music Team/Vocals`). Repeat for OR. The resolver only sees
    /// org-tree nodes that are on this service.
    #[arg(long = "in", value_name = "ID|NAME|PATH")]
    pub department: Vec<String>,
    /// Include each person's primary email in the output.
    #[arg(long)]
    pub email: bool,
    /// Id rendering: short (default), long, or hidden.
    #[arg(long = "id", value_enum, default_value_t = IdMode::Short, value_name = "MODE")]
    pub id_mode: IdMode,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ServicesSongUsageArgs {
    /// Inclusive start date (YYYY-MM-DD). Defaults to 12 months before --to.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub from: Option<String>,
    /// Inclusive end date (YYYY-MM-DD). Defaults to today (local time).
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub to: Option<String>,
    /// Only show songs used at most this many times (default: 2).
    #[arg(long, default_value_t = 2, value_name = "N")]
    pub max_uses: u32,
    /// Only show songs that have been led by a single person (ignores --max-uses).
    #[arg(long)]
    pub one_leader: bool,
}

#[derive(Debug, Args)]
pub struct ServicesListArgs {
    /// Inclusive start date (YYYY-MM-DD). Defaults to 6 months before --to.
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub from: Option<String>,
    /// Inclusive end date (YYYY-MM-DD). Defaults to today (local time).
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub to: Option<String>,
    /// Id rendering: short (default), long, or hidden.
    #[arg(long = "id", value_enum, default_value_t = IdMode::Short, value_name = "MODE")]
    pub id_mode: IdMode,
    /// Emit normalized JSON instead of text.
    #[arg(long)]
    pub json: bool,
}
