use super::block_position::BlockPosition;

pub enum RangeType {
    Cubic,
}

pub struct BlockPositionRange {
    start: BlockPosition,
    end: BlockPosition,
    current: BlockPosition,
    range_type: RangeType,
}

impl Iterator for BlockPositionRange {
    type Item = BlockPosition;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.range_type {
            RangeType::Cubic => self.next_cubic(),
        };

        if let Some(new_current) = next {
            self.current = new_current;
        }

        next
    }
}

impl BlockPositionRange {
    pub fn new(start: BlockPosition, end: BlockPosition, range_type: RangeType) -> Self {
        BlockPositionRange {
            start,
            end,
            range_type,
            current: start,
        }
    }

    fn next_cubic(&self) -> Option<BlockPosition> {
        if self.current == self.end {
            return None;
        }

        if self.current.x < self.end.x {
            return Some(self.current.offset(1, 0, 0));
        } else if self.current.x > self.end.x {
            return Some(self.current.offset(-1, 0, 0));
        }

        if self.current.y < self.end.y {
            return Some(self.current.offset(0, 1, 0));
        } else if self.current.y > self.end.y {
            return Some(self.current.offset(0, -1, 0));
        }

        if self.current.z < self.end.z {
            return Some(self.current.offset(0, 0, 1));
        } else if self.current.z > self.end.z {
            return Some(self.current.offset(0, 0, -1));
        }

        None
    }
}
