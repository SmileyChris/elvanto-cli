use crate::api::Client;
use crate::cli::PeopleListArgs;
use crate::domain::person::{person_tree, Person};
use crate::error::CliError;
use crate::output;
use crate::resolve;

pub async fn run(client: &Client, args: PeopleListArgs) -> Result<(), CliError> {
    let raws = client.list_all_people(&["departments"]).await?;
    let mut people: Vec<Person> = raws.into_iter().map(Into::into).collect();
    if !args.department.is_empty() {
        let tree = person_tree(&people);
        let resolved = resolve::resolve_all(&args.department, &tree)?;
        people.retain(|p| p.matches_department(&resolved));
    }
    if !args.json {
        people.retain(|p| p.status == "active");
    }
    people.sort_by(|a, b| a.name.cmp(&b.name));

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let res = if args.json {
        output::json::write_pretty(&mut lock, &people)
    } else {
        output::text::write_people(&mut lock, &people, args.id_mode)
    };
    res.map_err(|e| CliError::Io(format!("write error: {e}")))
}
