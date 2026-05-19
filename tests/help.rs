mod common;
use common::bin;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn shows_top_level_help() {
    bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("auth").and(contains("songs")));
}

#[test]
fn songs_subcommands_listed() {
    bin().args(["songs", "--help"]).assert().success().stdout(
        contains("categories")
            .and(contains("list"))
            .and(contains("show"))
            .and(contains("chart"))
            .and(contains("lyrics")),
    );
}
