enum StateFlags {
    Solid,
    Rotation,
}

impl StateFlags {
    fn least_significant_bit_pos(&self) -> u32 {
        match self {
            StateFlags::Solid => 0,
            StateFlags::Rotation => 1,
        }
    }

    fn mask(&self) -> u32 {
        match self {
            StateFlags::Solid => (1 << 1) - 1,
            StateFlags::Rotation => (1 << 2) - 1,
        }
    }

    pub fn get(&self, state: u32) -> u32 {
        state >> self.least_significant_bit_pos() & self.mask()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub id: u16,
    pub state: u32,
}

impl Block {
    pub fn is_solid(&self) -> bool {
        return StateFlags::Solid.get(self.state) != 0;
    }

    // pub fn facing_direction(&self) -> Direction {
    //     Direction::try_from(value)
    //     match StateFlags::Rotation.get(self.state) {

    //     }
    // }

    pub fn is_air(&self) -> bool {
        self.id == 0
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            id: Default::default(),
            state: Default::default(),
        }
    }
}
