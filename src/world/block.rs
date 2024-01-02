#[derive(Copy, Clone)]
pub struct Block {
    id: u16,
    state: u32,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            id: Default::default(),
            state: Default::default(),
        }
    }
}
