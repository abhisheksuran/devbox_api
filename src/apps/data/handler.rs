use crate::db::DefaultDB;
use crate::models::{Container, ContainerConfig};
use rusqlite::params;

pub async fn list_containers() -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;

    let mut stmt = conn.prepare("SELECT name, status, config FROM containers")?;
    let container_iter = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let status: String = row.get(1)?;
        let config_str: String = row.get(2)?;
        let config: ContainerConfig = serde_json::from_str(&config_str).unwrap();
        Ok(Container {
            name,
            status: Some(status),
            config,
        })
    })?;

    let containers: Result<Vec<Container>, _> = container_iter.collect();
    let json = serde_json::to_string(&containers?)?;
    Ok(json)
}

pub async fn create_container(input: Container) -> Result<(), Box<dyn std::error::Error>> {
    let ipt = Container::new(&input.name, input.status.as_deref(), input.clone());
    let config_json = serde_json::to_string(&input.config)?;
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "INSERT INTO containers (name, status, config) VALUES (?1, ?2, ?3)",
        [&ipt.name, &ipt.status.unwrap(), &config_json],
    )?;
    Ok(())
}

pub async fn fetch_container(name: String) -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    let mut stmt = conn.prepare("SELECT name, status, config FROM containers WHERE name = ?1")?;

    let container = stmt.query_row(params![name], |row| {
        let json_str: String = row.get(2)?;
        Ok(Container {
            name: row.get(0)?,
            status: row.get(1)?,
            config: serde_json::from_str(&json_str).unwrap(),
        })
    })?;
    let json = serde_json::to_string(&container)?;
    Ok(json)
}
