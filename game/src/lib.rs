use std::num::NonZeroU16;

use rand::prelude::*;

const MOVES: &'static [Move] = &[Move::Left, Move::Right, Move::Up, Move::Down];

#[derive(Clone, Copy, Debug)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

pub trait BoardShuffle {
    fn shuffle(&mut self, board: &mut Board);
}

pub struct DummyShuffle;

impl BoardShuffle for DummyShuffle {
    fn shuffle(&mut self, _board: &mut Board) {}
}

pub struct RandomShuffle {
    rng: ThreadRng,
}

impl RandomShuffle {
    pub fn new(rng: ThreadRng) -> Self {
        Self { rng }
    }
}

impl BoardShuffle for RandomShuffle {
    fn shuffle(&mut self, board: &mut Board) {
        let num_shuffle_moves = (board.size() as usize).pow(4);
        let mut i = 0;
        while i < num_shuffle_moves {
            let mv = *MOVES.choose(&mut self.rng).expect("random move");
            if board.move_once(mv) {
                i += 1;
            }
        }
    }
}

#[derive(PartialEq)]
pub struct Board {
    cells: Vec<Option<NonZeroU16>>,
    size: u8,
    free_cell_ix: usize,
}

impl Board {
    pub fn new(size: u8, shuffler: &mut dyn BoardShuffle) -> anyhow::Result<Self> {
        let num_cells = (size as u16) * (size as u16);
        let cells = (1..num_cells).chain(0..1).map(NonZeroU16::new).collect::<Vec<_>>();
        let free_cell_ix = cells.len() - 1;
        let mut board = Self {
            cells,
            size,
            free_cell_ix,
        };
        shuffler.shuffle(&mut board);
        Ok(board)
    }

    pub fn get(&self, row: u8, col: u8) -> Option<NonZeroU16> {
        self.cells[(row as usize) * (self.size as usize) + (col as usize)]
    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn move_once(&mut self, mv: Move) -> bool {
        // println!("Moving {mv:?}");
        use Move ::*;

        // When calculating target cell index it can become negative
        let free_cell_ix = self.free_cell_ix as isize;
        let size = self.size as isize;
        let target_cell_ix = match mv {
            Left => {
                let next_ix = free_cell_ix + 1;
                if next_ix % self.size as isize == 0 {
                    return false;
                }
                next_ix
            }
            Right => {
                if free_cell_ix % self.size as isize == 0 {
                    return false;
                }
                free_cell_ix - 1
            }
            Up => free_cell_ix + size,
            Down => free_cell_ix - size,
        };
        if target_cell_ix < 0 || target_cell_ix >= self.cells.len() as isize {
            return false;
        }
        // println!("Swapping {free_cell_ix} <-> {target_cell_ix}");
        self.cells.swap(self.free_cell_ix, target_cell_ix as usize);
        self.free_cell_ix = target_cell_ix as usize;

        true
    }

    pub fn move_many(&mut self, moves: &[Move]) -> usize {
        let mut successful_moves = 0;
        for &mv in moves {
            if self.move_once(mv) {
                successful_moves += 1;
            }
        }
        successful_moves
    }

    pub fn rows(&self) -> Vec<&[Option<NonZeroU16>]> {
        self.cells.chunks(self.size as usize).collect()
    }

    // Used in tests
    #[allow(dead_code)]
    fn to_rows(&self) -> Vec<Vec<Option<u16>>> {
        self.rows().iter()
            .map(|row| {
                row.iter()
                    .map(|cell| cell.map(NonZeroU16::get))
                    .collect()
            })
            .collect()
    }

    pub fn is_ordered(&self) -> bool {
        let mut prev_cell = None;
        for cell in self.cells.iter() {
            match (cell, prev_cell) {
                (Some(cell_val), Some(prev_cell_val)) => {
                    if cell_val <= prev_cell_val {
                        return false;
                    } else {
                        prev_cell = Some(cell_val);
                    }
                }
                (None, _) => continue,
                (Some(_), None) => {
                    prev_cell = cell.as_ref();
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;
    use super::{Board, DummyShuffle, Move};

    #[test]
    fn board_1x1() {
        let mut board = Board::new(1, &mut DummyShuffle).expect("board");
        assert_eq!(board.size(), 1);
        assert_eq!(&board.to_rows(), &[&[None]]);
        assert!(board.is_ordered());

        board.move_once(Move::Left);
        assert!(board.is_ordered());
        board.move_once(Move::Right);
        assert!(board.is_ordered());
        board.move_once(Move::Up);
        assert!(board.is_ordered());
        board.move_once(Move::Down);
        assert!(board.is_ordered());
    }

    #[test]
    fn board_4x4() {
        let mut board = Board::new(4, &mut DummyShuffle).expect("board");
        assert_eq!(board.size(), 4);
        let expected_rows = &[
            &[1.into(), 2.into(), 3.into(), 4.into()],
            &[5.into(), 6.into(), 7.into(), 8.into()],
            &[9.into(), 10.into(), 11.into(), 12.into()],
            &[13.into(), 14.into(), 15.into(), None],
        ];
        assert_eq!(
            &board.to_rows(),
            expected_rows
        );
        assert!(board.is_ordered());

        board.move_once(Move::Left);
        assert_eq!(
            &board.to_rows(),
            expected_rows
        );
        assert!(board.is_ordered());

        board.move_once(Move::Up);
        assert_eq!(
            &board.to_rows(),
            expected_rows
        );
        assert!(board.is_ordered());

        board.move_once(Move::Right);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 2.into(), 3.into(), 4.into()],
                &[5.into(), 6.into(), 7.into(), 8.into()],
                &[9.into(), 10.into(), 11.into(), 12.into()],
                &[13.into(), 14.into(), None, 15.into()],
            ]
        );
        assert!(board.is_ordered());

        // Check not crossing left border
        board.move_many(&[Move::Right, Move::Right, Move::Right]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 2.into(), 3.into(), 4.into()],
                &[5.into(), 6.into(), 7.into(), 8.into()],
                &[9.into(), 10.into(), 11.into(), 12.into()],
                &[None, 13.into(), 14.into(), 15.into()],
            ]
        );
        assert!(board.is_ordered());

        board.move_many(&[
            Move::Left, Move::Left, Move::Down, Move::Down, Move::Down, Move::Right, Move::Right
        ]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[None, 1.into(), 2.into(), 4.into()],
                &[5.into(), 6.into(), 3.into(), 8.into()],
                &[9.into(), 10.into(), 7.into(), 12.into()],
                &[13.into(), 14.into(), 11.into(), 15.into()],
            ]
        );
        assert!(!board.is_ordered());

        board.move_many(&[Move::Left, Move::Left, Move::Up, Move::Up, Move::Left]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 2.into(), 3.into(), 4.into()],
                &[5.into(), 6.into(), 7.into(), 8.into()],
                &[9.into(), 10.into(), 12.into(), None],
                &[13.into(), 14.into(), 11.into(), 15.into()],
            ]
        );
        assert!(!board.is_ordered());

        // Check not crossing right border
        board.move_once(Move::Left);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 2.into(), 3.into(), 4.into()],
                &[5.into(), 6.into(), 7.into(), 8.into()],
                &[9.into(), 10.into(), 12.into(), None],
                &[13.into(), 14.into(), 11.into(), 15.into()],
            ]
        );
        assert!(!board.is_ordered());

        board.move_many(&[Move::Right, Move::Up, Move::Left]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 2.into(), 3.into(), 4.into()],
                &[5.into(), 6.into(), 7.into(), 8.into()],
                &[9.into(), 10.into(), 11.into(), 12.into()],
                &[13.into(), 14.into(), 15.into(), None],
            ]
        );
        assert!(board.is_ordered());
    }

    #[test]
    fn board_255x255() {
        let board = Board::new(255, &mut DummyShuffle).expect("board");
        assert_eq!(board.size(), 255);
        assert_eq!(board.get(0, 0), NonZeroU16::new(1));
        assert_eq!(board.get(0, 1), NonZeroU16::new(2));
        assert_eq!(board.get(0, 254), NonZeroU16::new(255));
        assert_eq!(board.get(1, 0), NonZeroU16::new(256));
        assert_eq!(board.get(2, 0), NonZeroU16::new(511));
        assert_eq!(board.get(254, 253), NonZeroU16::new(65024));
        assert_eq!(board.get(254, 254), None);
        assert!(board.is_ordered());
    }
}
