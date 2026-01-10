use crate::apps::artifactory::Artifactory;
use crate::db::DefaultDB;
use crate::providers::ProviderEnum;
use rusqlite::params;

pub async fn get_provider_and_id(
    id: &String,
) -> Result<(ProviderEnum, String), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db().unwrap();
    let mut stmt = conn.prepare("SELECT provider, resource_id FROM containers WHERE id = ?")?;
    let result = stmt.query_row(params![id], |row| {
        let provider: String = row.get(0)?;
        let resource_id: String = row.get(1)?;
        Ok((ProviderEnum::from(provider), resource_id))
    })?;
    Ok(result)
}

pub async fn get_task_id(id: String) -> Result<String, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db().unwrap();
    let mut stmt = conn.prepare("SELECT task_id FROM containers WHERE id = ?")?;
    let task_id = stmt.query_row(params![id], |row| {
        let task_id: String = row.get(0)?;
        Ok(task_id)
    })?;
    Ok(task_id)
}

pub async fn insert_task(
    id: &str,
    provider: &str,
    status: &str,
    request: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = std::fs::read_to_string(format!("{}/devbox.json", request)).unwrap_or(request);

    let conn = DefaultDB::get_db()?;
    conn.execute(
        "INSERT INTO tasks (id, provider, status, request)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, provider, status, request],
    )?;
    Ok(())
}

pub async fn list_artifactory()
-> Result<Vec<(String, String, String, String, String, String)>, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    type ArtifactoryRow = Vec<(String, String, String, String, String, String)>;
    let mut stmt = conn.prepare(
        "SELECT provider, server, repository_name, username, password, config FROM artifactories",
    )?;
    let aartifactory_iter = stmt.query_map([], |row| {
        let provider: String = row.get(0)?;
        let server: String = row.get(1)?;
        let repository: String = row.get(2)?;
        let user: String = row.get(3)?;
        let password: String = row.get(4)?;
        let config: String = row.get(5)?;
        Ok((provider, server, repository, user, password, config))
    })?;

    let artifactories: Result<ArtifactoryRow, _> = aartifactory_iter.collect();
    match artifactories {
        Ok(c) => Ok(c),
        Err(_) => Err("Unable to fetch artifactories list from db".into()),
    }
}

pub async fn insert_artifactory(
    provider: &str,
    config: Artifactory,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "INSERT INTO artifactories (provider, server, repository_name, username, password)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            provider,
            config.server,
            config.repository_name,
            config.username,
            config.password
        ],
    )?;
    Ok(())
}

pub async fn update_artifactory(
    provider: &str,
    config: Artifactory,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "UPDATE artifactories SET  server = ?2, repository_name = ?3, username = ?4, password = ?5 WHERE provider = ?1",
        rusqlite::params![provider, config.server, config.repository_name, config.username, config.password],
    )?;
    Ok(())
}

pub async fn delete_artifactory(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "DELETE FROM artifactories WHERE provider = ?",
        params![provider],
    )?;
    Ok(())
}

pub async fn get_latest_config()
-> Result<Vec<(String, serde_json::Value, Option<Artifactory>)>, Box<dyn std::error::Error>> {
    let conn: rusqlite::Connection = DefaultDB::get_db()?;
    type DataRow = Vec<(String, serde_json::Value, Option<Artifactory>)>;
    let mut stmt = conn.prepare("SELECT p.name, p.config, a.server, a.repository_name, a.username, a.password  FROM providers as p LEFT JOIN artifactories as a ON p.name = a.provider")?;
    let container_iter = stmt.query_map([], |row| {
        let provider: String = row.get(0)?;
        let config: String = row.get(1)?;
        let config_json = serde_json::to_value(config).unwrap();
        let server: Option<String> = row.get(2).ok();
        let repository: Option<String> = row.get(3).ok();
        let mut artifactory: Option<Artifactory> = None;
        if server.is_some() && repository.is_some() {
            artifactory = Some(Artifactory {
                server: row.get(2)?,
                repository_name: row.get(3)?,
                username: row.get(4)?,
                password: row.get(5)?,
            })
        };

        Ok((provider, config_json, artifactory))
    })?;

    let provider_data: Result<DataRow, _> = container_iter.collect();
    match provider_data {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Unable to fetch container list from db: {}", e).into()),
    }
}

pub async fn list_containers()
-> Result<Vec<(i32, String, String, String, String, String)>, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    type ContainerRow = Vec<(i32, String, String, String, String, String)>;
    let mut stmt =
        conn.prepare("SELECT id, name, provider, status, resource_id, task_id FROM containers")?;
    let container_iter = stmt.query_map([], |row| {
        let container_id: i32 = row.get(0)?;
        let name: String = row.get(1)?;
        let provider: String = row.get(2)?;
        let status: String = row.get(3)?;
        let resource_id: String = row.get(4)?;
        let task_id: String = row.get(5)?;
        Ok((container_id, name, provider, status, resource_id, task_id))
    })?;

    let containers: Result<ContainerRow, _> = container_iter.collect();
    match containers {
        Ok(c) => Ok(c),
        Err(_) => Err("Unable to fetch container list from db".into()),
    }
    // let json = serde_json::to_string(&containers?)?;
    // Ok(json)
}
pub async fn get_container(id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    let mut stmt =
        conn.prepare("SELECT id, name, provider, status FROM containers WHERE id = ?1")?;

    let container_json = stmt.query_row(params![id], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let provider: String = row.get(2)?;
        let status: String = row.get(3)?;
        Ok(serde_json::json!({
            "id": id,
            "name": name,
            "provider": provider,
            "status": status,
        }))
    })?;
    Ok(container_json)
}

pub async fn insert_container(
    provider: &str,
    name: &str,
    status: &str,
    task_id: &str,
    resource_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "INSERT INTO containers (name, provider, status, task_id, resource_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, provider, status, task_id, resource_id,],
    )?;
    Ok(())
}

pub async fn delete_container(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute("DELETE FROM containers WHERE resource_id = ?", params![id])?;
    Ok(())
}

pub async fn update_container_resource_id(
    task_id: &str,
    resource_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "UPDATE containers SET resource_id = ?1 WHERE task_id = ?2",
        rusqlite::params![resource_id, task_id],
    )?;
    Ok(())
}

pub async fn update_container_status(
    id: &str,
    new_status: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "UPDATE containers SET status = ?1 WHERE resource_id = ?2",
        rusqlite::params![new_status, id],
    )?;
    Ok(())
}

pub async fn insert_provider(
    provider: &str,
    config: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "INSERT INTO providers (name, config)
         VALUES (?1, ?2)",
        params![provider, config],
    )?;
    Ok(())
}

pub async fn update_provider_db(
    provider: &str,
    config: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "UPDATE providers SET config = ?2 WHERE name = ?1",
        rusqlite::params![provider, config],
    )?;
    Ok(())
}

pub async fn delete_provider(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute("DELETE FROM providers WHERE name = ?", params![provider])?;
    Ok(())
}

pub async fn list_providers() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    type ProviderRow = Vec<(String, String)>;
    let mut stmt = conn.prepare("SELECT name, config FROM providers")?;
    let provider_iter = stmt.query_map([], |row| {
        let provider: String = row.get(0)?;
        let config: String = row.get(1)?;
        Ok((provider, config))
    })?;

    let providers: Result<ProviderRow, _> = provider_iter.collect();
    match providers {
        Ok(c) => Ok(c),
        Err(_) => Err("Unable to fetch providers list from db".into()),
    }
}

pub async fn list_builders()
-> Result<Vec<(String, bool, String, String)>, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    type BuilderRow = Vec<(String, bool, String, String)>;
    let mut stmt = conn.prepare("SELECT name, remote, builder, config FROM builders")?;
    let container_iter = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let remote: bool = row.get(1)?;
        let builder: String = row.get(2)?;
        let config: String = row.get(3)?;
        Ok((name, remote, builder, config))
    })?;

    let builders: Result<BuilderRow, _> = container_iter.collect();
    match builders {
        Ok(c) => Ok(c),
        Err(_) => Err("Unable to fetch builders list from db".into()),
    }
}

pub async fn get_builder(name: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    let mut stmt =
        conn.prepare("SELECT name, remote, builder, config FROM builders WHERE name = ?1")?;

    let builder_json = stmt.query_row(params![name], |row| {
        let name: String = row.get(0)?;
        let remote: bool = row.get(1)?;
        let builder: String = row.get(2)?;
        let config: String = row.get(3)?;
        Ok(serde_json::json!({

            "name": name,
            "remote": remote,
            "builder": builder,
            "config": config
        }))
    })?;
    Ok(builder_json)
}

pub async fn insert_builder(
    name: &str,
    remote: u16,
    builder: &str,
    config: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute(
        "INSERT INTO builders (name, remote, builder, config)
         VALUES (?1, ?2, ?3, ?4)",
        params![name, remote, builder, config],
    )?;
    Ok(())
}

pub async fn delete_builder(builder: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = DefaultDB::get_db()?;
    conn.execute("DELETE FROM builders WHERE name = ?", params![builder])?;
    Ok(())
}
