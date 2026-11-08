#[ocamljs_bindgen]
impl Todo {
    #[ocamljs_bindgen(throws)]
    pub fn checked_add(&mut self, title: String) -> Result<u32, TaskError> {
        self.add(title)
    }
    pub fn into_count(self) -> u32 {
        self.tasks.len() as u32
    }
}
