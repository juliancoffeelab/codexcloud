use ocamljs_bindgen::ocamljs_bindgen;
#[ocamljsify::ocamljsify]
#[derive(Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub done: bool,
}
#[ocamljsify::ocamljsify]
#[derive(Debug, PartialEq)]
pub enum TaskError {
    EmptyTitle,
    UnknownTask(u32),
}
#[ocamljs_bindgen(opaque)]
pub struct Todo {
    tasks: Vec<Task>,
    next: u32,
}
#[ocamljs_bindgen]
impl Todo {
    pub fn create() -> Todo {
        Todo {
            tasks: vec![],
            next: 0,
        }
    }
    pub fn add(&mut self, title: String) -> Result<u32, TaskError> {
        if title.trim().is_empty() {
            return Err(TaskError::EmptyTitle);
        }
        self.next += 1;
        self.tasks.push(Task {
            id: self.next,
            title,
            done: false,
        });
        Ok(self.next)
    }
    pub fn toggle(&mut self, id: u32) -> Result<(), TaskError> {
        let t = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(TaskError::UnknownTask(id))?;
        t.done = !t.done;
        Ok(())
    }
    pub fn remove(&mut self, id: u32) -> Result<(), TaskError> {
        let index = self
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or(TaskError::UnknownTask(id))?;
        self.tasks.remove(index);
        Ok(())
    }
    pub fn tasks(&self) -> Vec<Task> {
        self.tasks.clone()
    }
    pub fn remaining(&self) -> u32 {
        self.tasks.iter().filter(|t| !t.done).count() as u32
    }
}

#[cfg(feature = "conformance")]
include!("../../tests/rust/conformance.rs");
