use std::path::Path;

use rusqlite::{Connection, Result};

pub fn init_db(path: &Path) -> Connection {
    let conn = Connection::open(path).expect("Failed to open database");
    conn.execute_batch(include_str!("schema.sql")).unwrap();
    conn
}

pub struct Project {
    pub id: i32,
    pub name: String,
    pub target_hours: Option<f32>,
    pub parent: Option<i32>,
}

pub fn get_projects(db: &Connection) -> Result<Vec<Project>> {
    let mut stmt = db.prepare("SELECT id, name, target_hours, parent FROM projects")?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                target_hours: row.get(2)?,
                parent: row.get(3)?,
            })
        })?
        .collect();
    projects
}

pub fn add_project(db: &Connection, project: &Project) -> Result<usize> {
    db.execute(
        "INSERT INTO projects (name, target_hours, parent) VALUES (?1, ?2, ?3)",
        (&project.name, project.target_hours, project.parent),
    )
}

pub fn update_project(db: &Connection, project: &Project) -> Result<usize> {
    db.execute(
        "UPDATE projects
        SET name = ?2, target_hours = ?3, parent = ?4
        WHERE id = ?1",
        (
            project.id,
            &project.name,
            project.target_hours,
            project.parent,
        ),
    )
}

pub fn delete_project(db: &Connection, project: &Project) -> Result<usize> {
    db.execute("DELETE FROM projects WHERE id = ?1", (project.id,))
}
