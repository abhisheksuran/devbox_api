use crate::common::DefaultDB;
use serde::{Serialize, Deserialize};
use rusqlite::params;

#[derive(Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct ContainerConfig {
    image : String,
    features: Option<Vec<String>>
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Container {
    name: String,
    status: Option<String>,
    config: ContainerConfig

}

impl Container {
    pub fn new(name: &str, status: Option<&str>, container: Container) -> Self {
        Self {
            name: name.to_string(),
            status: match status {
                Some(val) => Some(val.to_string()),
                _ => Some("Creating".to_string())
            },
            ..container

        }
    }
}



pub fn list_containers() -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;

    let mut stmt = conn.prepare("SELECT name, status, config FROM containers")?;
    let container_iter = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let status: String = row.get(1)?;
        let config_str: String = row.get(2)?;
        let config: ContainerConfig = serde_json::from_str(&config_str).unwrap();
        Ok(Container { name: name, status: Some(status), config: config})
    })?;

    let containers: Result<Vec<Container>, _> = container_iter.collect();
    let json = serde_json::to_string(&containers?)?;
    Ok(json)
}


pub fn create_container(input: Container) -> Result<(), Box<dyn std::error::Error>> {
    let ipt = Container::new(&input.name, input.status.as_deref(), input.clone());
    let config_json = serde_json::to_string(&input.config)?;
    let conn = DefaultDB::get_db()?;
    conn.execute("INSERT INTO containers (name, status, config) VALUES (?1, ?2, ?3)",[&ipt.name, &ipt.status.unwrap(), &config_json],)?;
    Ok(())
}

pub fn fetch_container(name: String) -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    let mut stmt = conn.prepare("SELECT name, status, config FROM containers WHERE name = ?1")?;

    let container =  stmt.query_row(params![name], |row| {
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