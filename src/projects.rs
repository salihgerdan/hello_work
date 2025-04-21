use std::ops::Index;

use rusqlite::Connection;

use crate::db::{self, Project};

// A struct for caching project data to minimize DB access
pub struct Projects {
    projects: Vec<Project>,
    active: Option<usize>,
}

impl Projects {
    pub fn new(conn: &Connection) -> Self {
        let mut p = Projects {
            projects: vec![],
            active: None,
        };
        p.fetch(conn);
        p
    }
    fn fetch(&mut self, conn: &Connection) {
        self.projects.truncate(0);
        self.projects
            .append(&mut db::get_projects(conn).expect("Failed to fetch projects"));
    }
    pub fn get(&self) -> &[Project] {
        &self.projects
    }
    pub fn add(&mut self, project: Project, conn: &Connection) {
        db::add_project(conn, &project);
        self.projects.push(project);
    }
    pub fn set_active(&mut self, id: i32) {
        self.active = self.projects.iter().position(|x| x.id == id);
    }
    pub fn get_active(&self) -> Option<&Project> {
        self.active.and_then(|x| self.projects.get(x))
    }
}
