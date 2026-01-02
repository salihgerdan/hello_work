use std::path::Path;

use chrono::{Local, NaiveDate, NaiveTime, TimeDelta};
use rusqlite::{Connection, Result};

use crate::{projects::Project, todo_tasks::TodoTask};

pub fn init_db(path: &Path) -> Connection {
    let conn = Connection::open(path).expect("Failed to open database");
    conn.execute_batch(include_str!("schema.sql")).unwrap();
    conn
}

pub fn get_projects(db: &Connection) -> Result<Vec<Project>> {
    let mut stmt = db.prepare(
        "WITH RECURSIVE project_aggregates AS (
            SELECT
                p.id,
                p.parent,
                p.id AS root_project_id,
                COALESCE(wt.duration, 0) AS duration_contribution
            FROM
                projects p
            LEFT JOIN
                work_totals wt ON p.id = wt.project_id
            
            UNION ALL
            
            SELECT
                p.id,
                p.parent,
                h.root_project_id,
                h.duration_contribution
            FROM
                projects p
            INNER JOIN
                project_aggregates h ON p.id = h.parent
        )

        SELECT
            p.id,
            p.name,
            p.target_hours,
            p.parent,
            GROUP_CONCAT(c.id) as children,
            total_duration
        FROM
            projects p
        LEFT JOIN
            (
                SELECT
                    id,
                    SUM(duration_contribution) AS total_duration
                FROM
                    project_aggregates
                GROUP BY
                    id
            ) a ON p.id = a.id
        LEFT JOIN
            projects c ON c.parent = p.id
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
    // permanently delete if the project has no desdendants or any recorded session
    let recorded_session_count: usize = db.query_row(
        "SELECT COUNT(time_start) FROM work WHERE project_id = ?1",
        (id,),
        |row| row.get(0),
    )?;
    let direct_child_count: usize = db.query_row(
        "SELECT COUNT(id) FROM projects WHERE parent = ?1",
        (id,),
        |row| row.get(0),
    )?;
    if recorded_session_count == 0 && direct_child_count == 0 {
        // clean up tasks associated as well when deleting
        db.execute("DELETE FROM tasks WHERE project_id = ?1", (id,))?;
        db.execute("DELETE FROM projects WHERE id = ?1;", (id,))
    } else {
        db.execute("UPDATE projects SET archived = 1 WHERE id = ?1", (id,))
    }
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

pub fn get_work_hours_for_day(
    db: &Connection,
    day: &NaiveDate,
    config_offset_hours: u32,
) -> Result<f32> {
    //let local_offset = Local::now().offset().clone();
    let day_start = day
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .and_local_timezone(Local::now().timezone())
        .unwrap()
        .checked_add_signed(TimeDelta::hours((config_offset_hours % 24) as i64))
        .unwrap();
    let next_day_start = day_start.checked_add_signed(TimeDelta::hours(24)).unwrap();
    db.query_row::<Option<f32>, _, _>(
        "SELECT SUM(duration)
        FROM work
        WHERE time_start >= ?1 AND time_start < ?2",
        (day_start.timestamp(), next_day_start.timestamp()),
        |row| row.get(0),
    )
    .map(|secs| secs.unwrap_or(0.0) / (60.0 * 60.0))
}

pub fn get_tasks(db: &Connection, project_id: Option<usize>) -> Result<Vec<TodoTask>> {
    let mut stmt = db.prepare(
        "SELECT
            id,
            name
        FROM
            tasks
        WHERE
            project_id = ?1",
    )?;
    let mut stmt_null = db.prepare(
        "SELECT
            id,
            name
        FROM
            tasks
        WHERE
            project_id IS NULL",
    )?;
    if let Some(project_id) = project_id {
        stmt.query_map((project_id,), |row| {
            Ok(TodoTask {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect()
    } else {
        stmt_null
            .query_map((), |row| {
                Ok(TodoTask {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect()
    }
}

pub fn add_task(db: &Connection, name: String, project_id: Option<usize>) -> Result<usize> {
    db.query_row(
        "INSERT INTO tasks (name, project_id) VALUES (?1, ?2) RETURNING id",
        (name, project_id),
        |row| row.get(0),
    )
}

pub fn update_task(db: &Connection, id: usize, name: String) -> Result<usize> {
    db.execute(
        "UPDATE tasks
        SET name = ?2
        WHERE id = ?1",
        (id, name),
    )
}

pub fn delete_task(db: &Connection, id: usize) -> Result<usize> {
    db.execute("DELETE FROM tasks WHERE id = ?1", (id,))
}
