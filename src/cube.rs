// how am i going to represent this cube?
// bitmaps for space and time efficiency - but still how?

// direction-color map: U=W F=G L=O
// direction ranking - U/D > F/B > L/R

// edges numbered (4 bits 0-B) WG to WR (0 - 3), GO to GR (4 - 7) and YG to YR (8 - B)
// edge orientation checking (1 bit) - color ranking and face ranking are in order
// 5 * 12 = 60 bits for all edges - 4 unused

// corners numbered (3 bits) clockwise UFL to UFR and DFL to DFR
// orientation check (2 bits) - 1 of 3 orientations
// 00 - W/Y is U/D
// 01 - W/Y is F/B
// 10 - W/Y is L/R
// 5 * 8 = 40 bits for all corners - 24 unused

use bevy::ecs::resource::Resource;

#[derive(Clone, Copy)]
pub enum MoveFace {
    R,
    L,
    U,
    F,
    B,
    D,
}

#[derive(Clone, Copy)]
pub struct Move {
    pub face: MoveFace,
    pub is_prime: bool,
    pub is_double: bool,
}

impl Move {
    pub fn new(face: MoveFace, is_prime: bool, is_double: bool) -> Self {
        Self {
            face,
            is_prime,
            is_double,
        }
    }

    // pub fn new_single(face: MoveFace) -> Self {
    //     Self {
    //         face,
    //         is_prime: false,
    //         is_double: false,
    //     }
    // }

    // pub fn new_prime(face: MoveFace) -> Self {
    //     Self {
    //         face,
    //         is_prime: true,
    //         is_double: false,
    //     }
    // }

    // pub fn new_double(face: MoveFace) -> Self {
    //     Self {
    //         face,
    //         is_prime: false,
    //         is_double: true,
    //     }
    // }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeState(u64);

impl EdgeState {
    // given an index, returns the id of the edge
    fn get(&self, pos: usize) -> u64 {
        (self.0 & (0b11111 << pos * 5)) >> pos * 5
    }

    // also returns replaced edge
    fn set(&mut self, pos: usize, edge: u64) -> u64 {
        let ret = self.get(pos);

        // clear edge
        self.0 = self.0 & !(0b11111 << pos * 5);

        // set edge
        self.0 = self.0 | (edge << pos * 5);

        ret
    }

    // Cycles p[0]→p[1]→p[2]→p[3]→p[0], optionally flipping orientation of all 4
    fn cycle4(&mut self, p: [usize; 4], flip: bool) {
        let tmp = self.set(p[1], self.get(p[0]));
        let tmp = self.set(p[2], tmp);
        let tmp = self.set(p[3], tmp);
        self.set(p[0], tmp);
        if flip {
            for &i in &p {
                let v = self.get(i);
                self.set(i, v ^ 0b10000);
            }
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CornerState(u64);

const SOLVED_EDGE: EdgeState =
    EdgeState(0b_01011_01010_01001_01000_00111_00110_00101_00100_00011_00010_00001_00000);
const SOLVED_CORNER: CornerState = CornerState(0b_00111_00110_00101_00100_00011_00010_00001_00000);

#[derive(Resource)]
pub struct Cube {
    edge: EdgeState,
    corner: CornerState,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            edge: SOLVED_EDGE,
            corner: SOLVED_CORNER,
        }
    }
}

impl Cube {
    pub fn is_solved(self) -> bool {
        self.edge == SOLVED_EDGE && self.corner == SOLVED_CORNER
    }

    pub fn get_edge(&self, pos: usize) -> u64 {
        self.edge.get(pos)
    }

    pub fn make_move(&mut self, m: Move) {
        let (mut cycle, flip) = match m.face {
            MoveFace::U => ([0, 1, 2, 3], false),
            MoveFace::D => ([11, 10, 9, 8], false),
            MoveFace::R => ([3, 6, 11, 7], false),
            MoveFace::L => ([1, 4, 9, 5], false),
            MoveFace::F => ([0, 7, 8, 4], true),
            MoveFace::B => ([2, 5, 10, 6], true),
        };

        if m.is_prime {
            cycle.reverse();
        }

        let times = if m.is_double { 2 } else { 1 };
        for _ in 0..times {
            self.edge.cycle4(cycle, flip);
        }
    }
}
