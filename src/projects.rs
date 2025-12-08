use std::iter;

use rusqlite::Connection;

use crate::db::{self, Project};

// A struct for caching project data to minimize DB access
pub struct Projects {
    projects: Vec<Project>,
    active: Option<usize>,
    edited: Option<Project>,
}

impl Projects {
    pub fn new(conn: &Connection) -> Self {
        let mut p = Projects {
            projects: vec![],
            active: None,
            edited: None,
        };
        p.fetch(conn);
        p
    }
    fn fetch(&mut self, conn: &Connection) {
        self.projects.truncate(0);
        self.projects
            .append(&mut db::get_projects(conn).expect("Failed to fetch projects"));
    }
    pub fn get_all(&self) -> &[Project] {
        &self.projects
    }
    pub fn get_all_tree_style(&self) -> Vec<(usize, &Project)> {
        fn recurse<'a>(
            project: &'a Project,
            all_projects: &'a [Project],
            depth: usize,
        ) -> Vec<(usize, &'a Project)> {
            iter::once((depth, project))
                .chain(
                    project
                        .children
                        .iter()
                        .flat_map(|id| all_projects.iter().find(|p| p.id == *id))
                        .flat_map(|p| recurse(p, all_projects, depth + 1)),
                )
                .collect()
        }
        let all_projects = self.get_all();
        all_projects
            .iter()
            .filter(|p| p.parent.is_none())
            .flat_map(|p| recurse(p, all_projects, 0))
            .collect()
    }
    /*pub fn get(&self, id: usize) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }*/
    pub fn add(&mut self, parent: Option<usize>, conn: &Connection) {
        let id = db::add_project(conn, parent).expect("Failed to add project");
        self.fetch(conn);
        self.initiate_edit(Some(id));
    }
    pub fn set_active(&mut self, id: Option<usize>) {
        self.active = id;
    }
    pub fn get_active(&self) -> Option<usize> {
        self.active
    }
    pub fn get_active_project(&self) -> Option<&Project> {
        self.active
            .and_then(|x| self.projects.iter().find(|p| p.id == x))
    }
    pub fn initiate_edit(&mut self, id: Option<usize>) {
        self.edited = id
            .and_then(|id| self.projects.iter().find(|p| p.id == id))
            .cloned();
    }
    pub fn finish_edit(&mut self, conn: &Connection) {
        if let Some(edited) = self.edited.as_ref() {
            db::update_project(conn, edited).expect("Failed to update project");
        }
        self.edited = None;
        self.fetch(conn);
    }
    pub fn get_edited(&self) -> Option<&Project> {
        self.edited.as_ref()
    }
    pub fn get_edited_id(&self) -> Option<usize> {
        self.edited.as_ref().map(|p| p.id)
    }
    pub fn set_edited_name(&mut self, name: String) {
        if let Some(edited) = self.edited.as_mut() {
            edited.name = name;
        }
    }
    pub fn archive_edited_item(&mut self, conn: &Connection) {
        if let Some(edited) = self.edited.as_ref() {
            db::archive_project(conn, edited.id).expect("Failed to archive project");
        }
        self.edited = None;
        self.fetch(conn);
    }
}
