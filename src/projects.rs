use rusqlite::Connection;
use std::fmt::Display;
use std::iter;

use crate::db;

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

// A struct for caching project data to minimize DB access
pub struct Projects {
    projects: Vec<Project>,
    active: Option<usize>,
    edited: Option<Project>,
}

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
    pub fn fetch(&mut self, conn: &Connection) {
        self.projects.truncate(0);
        self.projects
            .append(&mut db::get_projects(conn).expect("Failed to fetch projects"));
    }
    pub fn get_all_tree_style(&self) -> Vec<(usize, &Project)> {
        self.projects
            .iter()
            .filter(|p| p.parent.is_none())
            .flat_map(|p| recurse(p, &self.projects, 0))
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
            // archive children too when parent is archived
            let hierarchy = recurse(edited, &self.projects, 0);
            // reverse to aid the removal, as only leaf nodes can be deleted
            // any node with children or recorded session will be archived
            for (_depth, p) in hierarchy.into_iter().rev() {
                db::archive_project(conn, p.id).expect("Failed to archive project");
            }
        }
        self.edited = None;
        self.fetch(conn);
    }
}
