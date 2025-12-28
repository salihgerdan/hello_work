use rusqlite::Connection;
use std::fmt::Display;

use crate::db;

#[derive(Clone, Debug, Default)]
pub struct TodoTask {
    pub id: usize,
    pub name: String,
}

impl Display for TodoTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TodoTasks {
    project_id: Option<usize>,
    tasks: Vec<TodoTask>,
}

impl TodoTasks {
    pub fn new(conn: &Connection, project_id: Option<usize>) -> Self {
        let mut t = TodoTasks {
            project_id,
            tasks: vec![],
        };
        t.fetch(conn);
        t
    }
    pub fn switch_project(&mut self, conn: &Connection, project_id: Option<usize>) {
        self.project_id = project_id;
        self.fetch(conn);
    }
    pub fn fetch(&mut self, conn: &Connection) {
        self.tasks.truncate(0);
        self.tasks
            .append(&mut db::get_tasks(conn, self.project_id).expect("Failed to fetch tasks"));
    }
    pub fn get_all(&self) -> &Vec<TodoTask> {
        &self.tasks
    }
    pub fn add(&mut self, name: String, project_id: Option<usize>, conn: &Connection) {
        let _id = db::add_task(conn, name, project_id).expect("Failed to add task");
        self.fetch(conn);
    }
    pub fn edit(&mut self, id: usize, name: String, conn: &Connection) {
        db::update_task(conn, id, name).expect("Failed to edit task");
        self.fetch(conn);
    }
    pub fn delete(&mut self, id: usize, conn: &Connection) {
        db::delete_task(conn, id).expect("Failed to delete task");
        self.fetch(conn);
    }
}
