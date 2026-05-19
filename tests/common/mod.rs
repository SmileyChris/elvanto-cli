use assert_cmd::Command;

pub fn bin() -> Command {
    Command::cargo_bin("elvanto").unwrap()
}

pub async fn mock_server() -> wiremock::MockServer {
    wiremock::MockServer::start().await
}
