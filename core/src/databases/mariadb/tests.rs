use std::env;

use anyhow::Result;
use dotenv::dotenv;
use log::LevelFilter;

use crate::databases::mariadb::{tools::MariaDBTools, version::MariaDBVersion};

struct ConnectionOptions {
    host: String,
    port: u16,
    user: String,
    password: String,
    db_name: String,
}

fn get_connection_options() -> Result<ConnectionOptions> {
    dotenv().ok();

    let port: u16 = env::var("MARIADB_PORT").unwrap_or("0".into()).parse()?;

    Ok(ConnectionOptions {
        host: env::var("MARIADB_HOST").unwrap_or_default(),
        password: env::var("MARIADB_PASSWORD").unwrap_or_default(),
        user: env::var("MARIADB_USERNAME").unwrap_or_default(),
        db_name: env::var("MARIADB_NAME").unwrap_or_default(),
        port,
    })
}

fn initialize_test() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();
}

#[test]
fn test_mariadb_tools_instantiation() {
    initialize_test();
    let mariadb_tools = MariaDBTools::default();
    assert_eq!(mariadb_tools.version, MariaDBVersion::V11_2);
}

#[tokio::test]
async fn test_mariadb_version() {
    initialize_test();
    let options = get_connection_options().expect("Failed to get connection options");
    let mariadb_tools = MariaDBTools::default();
    let version = mariadb_tools
        .get_version(
            &options.db_name,
            &options.host,
            options.port,
            &options.user,
            Some(options.password.as_str()),
        )
        .await
        .expect("Unable to check MariaDB connection");

    println!("{:?}", version);
}

#[tokio::test]
async fn test_mariadb_connection() {
    initialize_test();
    let options = get_connection_options().expect("Failed to get connection options");
    let mariadb_tools = MariaDBTools::default();
    let is_connected = mariadb_tools
        .is_connected(
            &options.db_name,
            &options.host,
            options.port,
            &options.user,
            Some(options.password.as_str()),
        )
        .await
        .expect("Unable to check MariaDB connection");

    assert!(is_connected);
}

#[tokio::test]
async fn test_mariadb_dump() {
    initialize_test();
    let options = get_connection_options().expect("Failed to get connection options");
    let mariadb_tools = MariaDBTools::with_detected_version(
        &options.db_name,
        &options.host,
        options.port,
        &options.user,
        Some(options.password.as_str()),
    )
    .await
    .expect("Failed to initialize MariaDBTools");

    let bytes = mariadb_tools
        .dump(
            &options.db_name,
            &options.host,
            options.port,
            &options.user,
            Some(options.password.as_str()),
            None,
        )
        .await
        .expect("Unable to check MariaDB connection");

    assert!(bytes.len() > 0);
}
