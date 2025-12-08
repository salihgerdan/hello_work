use std::{fmt::Display, path::Path};

use chrono::{NaiveDate, NaiveTime};
use rusqlite::{Connection, Result};

pub fn init_db(path: &Path) -> Connection {
    let conn = Connection::open(path).expect("Failed to open database");
    conn.execute_batch(include_str!("schema.sql")).unwrap();
    conn
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct Project {
    pub id: usize,
    pub name: String,
    pub target_hours: Option<f32>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub total_hours: f32,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

pub fn get_projects(db: &Connection) -> Result<Vec<Project>> {
    let mut stmt = db.prepare(
        "SELECT 
                p.id,
                p.name,
                p.target_hours,
                p.parent,
                GROUP_CONCAT(c.id) AS children,
                -- Use COALESCE to treat NULL as 0 and SUM the parent's work and the children's work
                COALESCE(parent_work.duration, 0) + COALESCE(child_work.total_children_duration, 0) AS total_duration
            FROM 
                projects p
            LEFT JOIN
                -- Calculate the work hours for the parent project
                (
                    SELECT
                        project_id,
                        SUM(duration) AS duration
                    FROM
                        work
                    GROUP BY
                        project_id
                ) parent_work ON p.id = parent_work.project_id
            LEFT JOIN
                -- Calculate the total work hours for all CHILDREN
                (
                    SELECT
                        c_work.parent AS parent_id,
                        SUM(w.duration) AS total_children_duration
                    FROM
                        projects c_work
                    INNER JOIN -- Use INNER JOIN here, we only care about children with work
                        work w ON c_work.id = w.project_id
                    WHERE
                        c_work.parent IS NOT NULL -- Only look at projects that are children
                    GROUP BY
                        c_work.parent
                ) child_work ON p.id = child_work.parent_id
            LEFT JOIN 
                projects c ON p.id = c.parent
            WHERE
                p.archived = 0
            GROUP BY 
                p.id;",
    )?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                target_hours: row.get(2)?,
                parent: row.get(3)?,
                children: row
                    .get::<_, Option<String>>(4)?
                    .map(|s| s.split(",").map(|x| x.parse().unwrap()).collect())
                    .unwrap_or_default(),
                total_hours: row.get::<_, f32>(5)? / (60.0 * 60.0),
            })
        })?
        .collect();
    projects
}

pub fn add_project(db: &Connection, parent: Option<usize>) -> Result<usize> {
    db.query_row(
        "INSERT INTO projects (name, parent) VALUES (?1, ?2) RETURNING id",
        ("", parent),
        |row| row.get(0),
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

pub fn archive_project(db: &Connection, id: usize) -> Result<usize> {
    db.execute(
        "UPDATE projects
        SET archived = 1
        WHERE id = ?1",
        (id,),
    )
}

pub struct WorkSession {
    pub time_start: u64,
    pub duration: u64,
    pub project_id: Option<usize>,
}

pub fn add_work_session(db: &Connection, work_session: &WorkSession) -> Result<usize> {
    db.execute(
        "INSERT INTO work (time_start, duration, project_id) VALUES (?1, ?2, ?3)",
        (
            work_session.time_start,
            work_session.duration,
            work_session.project_id,
        ),
    )
}

pub fn get_work_hours_for_day(db: &Connection, day: &NaiveDate) -> Result<f32> {
    let day_start = day
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_utc()
        .timestamp();
    let next_day_start = day
        .succ_opt()
        .unwrap()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_utc()
        .timestamp();
    db.query_row::<Option<f32>, _, _>(
        "SELECT SUM(duration)
        FROM work
        WHERE time_start >= ?1 AND time_start < ?2",
        (day_start, next_day_start),
        |row| row.get(0),
    )
    .map(|secs| secs.unwrap_or(0.0) / (60.0 * 60.0))
}
