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
