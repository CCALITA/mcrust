use glam::IVec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    Up = 0,    // +Y
    Down = 1,  // -Y
    North = 2, // -Z
    South = 3, // +Z
    East = 4,  // +X
    West = 5,  // -X
}

impl Direction {
    pub const ALL: [Direction; 6] = [
        Direction::Up,
        Direction::Down,
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];

    pub fn normal(self) -> IVec3 {
        match self {
            Direction::Up => IVec3::Y,
            Direction::Down => IVec3::NEG_Y,
            Direction::North => IVec3::NEG_Z,
            Direction::South => IVec3::Z,
            Direction::East => IVec3::X,
            Direction::West => IVec3::NEG_X,
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}
