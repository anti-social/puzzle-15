use std::marker::PhantomData;
use std::num::NonZeroU16;

pub trait Shuffle {
    type Item;

    fn shuffle(&mut self, data: &mut Vec<Self::Item>);
}

#[derive(Default)]
pub struct DummyShuffle<T> {
    _marker: PhantomData<T>,
}

impl<T> Shuffle for DummyShuffle<T> {
    type Item = T;

    fn shuffle(&mut self, _data: &mut Vec<Self::Item>) {}
}

#[derive(Clone, Copy, Debug)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

pub struct Board {
    cells: Vec<Option<NonZeroU16>>,
    size: u8,
    free_cell_ix: usize,
}

impl Board {
    pub fn new(size: u8, shuffler: &mut impl Shuffle<Item=Option<NonZeroU16>>) -> anyhow::Result<Self> {
        let num_cells = (size as u16) * (size as u16);
        let mut cells = (0..num_cells).map(NonZeroU16::new).collect();
        shuffler.shuffle(&mut cells);
        Ok(Self {
            cells,
            size,
            free_cell_ix: 0,
        })
    }

    pub fn get(&self, row: u8, col: u8) -> Option<NonZeroU16> {
        self.cells[(row as usize) * (self.size as usize) + (col as usize)]
    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn move_once(&mut self, mv: Move) {
        // println!("Moving {mv:?}");
        use Move ::*;

        // When calculating target cell index it can become negative
        let free_cell_ix = self.free_cell_ix as isize;
        let size = self.size as isize;
        let target_cell_ix = match mv {
            Left => {
                let next_ix = free_cell_ix + 1;
                if next_ix % self.size as isize == 0 {
                    return;
                }
                next_ix
            }
            Right => {
                if free_cell_ix % self.size as isize == 0 {
                    return;
                }
                free_cell_ix - 1
            }
            Up => free_cell_ix + size,
            Down => free_cell_ix - size,
        };
        if target_cell_ix < 0 || target_cell_ix >= self.cells.len() as isize {
            return;
        }
        // println!("Swapping {free_cell_ix} <-> {target_cell_ix}");
        self.cells.swap(self.free_cell_ix, target_cell_ix as usize);
        self.free_cell_ix = target_cell_ix as usize;
    }

    pub fn move_many(&mut self, moves: &[Move]) {
        for &mv in moves {
            self.move_once(mv);
        }
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
        let mut board = Board::new(1, &mut DummyShuffle::default()).expect("board");
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
        let mut board = Board::new(4, &mut DummyShuffle::default()).expect("board");
        assert_eq!(board.size(), 4);
        let expected_rows = &[
            &[None, 1.into(), 2.into(), 3.into()],
            &[4.into(), 5.into(), 6.into(), 7.into()],
            &[8.into(), 9.into(), 10.into(), 11.into()],
            &[12.into(), 13.into(), 14.into(), 15.into()],
        ];
        assert_eq!(
            &board.to_rows(),
            expected_rows
        );
        assert!(board.is_ordered());

        board.move_once(Move::Right);
        assert_eq!(
            &board.to_rows(),
            expected_rows
        );
        assert!(board.is_ordered());

        board.move_once(Move::Down);
        assert_eq!(
            &board.to_rows(),
            expected_rows
        );
        assert!(board.is_ordered());

        board.move_once(Move::Left);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), None, 2.into(), 3.into()],
                &[4.into(), 5.into(), 6.into(), 7.into()],
                &[8.into(), 9.into(), 10.into(), 11.into()],
                &[12.into(), 13.into(), 14.into(), 15.into()],
            ]
        );
        assert!(board.is_ordered());

        // Check not crossing right border
        board.move_many(&[Move::Left, Move::Left, Move::Left]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 2.into(), 3.into(), None],
                &[4.into(), 5.into(), 6.into(), 7.into()],
                &[8.into(), 9.into(), 10.into(), 11.into()],
                &[12.into(), 13.into(), 14.into(), 15.into()],
            ]
        );
        assert!(board.is_ordered());

        board.move_many(&[Move::Right, Move::Right, Move::Up, Move::Up, Move::Up, Move::Left, Move::Left]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 5.into(), 2.into(), 3.into()],
                &[4.into(), 9.into(), 6.into(), 7.into()],
                &[8.into(), 13.into(), 10.into(), 11.into()],
                &[12.into(), 14.into(), 15.into(), None],
            ]
        );
        assert!(!board.is_ordered());

        board.move_many(&[Move::Right, Move::Right, Move::Down, Move::Down, Move::Right]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 5.into(), 2.into(), 3.into()],
                &[None, 4.into(), 6.into(), 7.into()],
                &[8.into(), 9.into(), 10.into(), 11.into()],
                &[12.into(), 13.into(), 14.into(), 15.into()],
            ]
        );
        assert!(!board.is_ordered());

        // Check not crossing left border
        board.move_once(Move::Right);
        assert_eq!(
            &board.to_rows(),
            &[
                &[1.into(), 5.into(), 2.into(), 3.into()],
                &[None, 4.into(), 6.into(), 7.into()],
                &[8.into(), 9.into(), 10.into(), 11.into()],
                &[12.into(), 13.into(), 14.into(), 15.into()],
            ]
        );
        assert!(!board.is_ordered());

        board.move_many(&[Move::Left, Move::Down, Move::Right]);
        assert_eq!(
            &board.to_rows(),
            &[
                &[None, 1.into(), 2.into(), 3.into()],
                &[4.into(), 5.into(), 6.into(), 7.into()],
                &[8.into(), 9.into(), 10.into(), 11.into()],
                &[12.into(), 13.into(), 14.into(), 15.into()],
            ]
        );
        assert!(board.is_ordered());
    }

    #[test]
    fn board_255x255() {
        let board = Board::new(255, &mut DummyShuffle::default()).expect("board");
        assert_eq!(board.size(), 255);
        assert_eq!(board.get(0, 0), None);
        assert_eq!(board.get(0, 1), NonZeroU16::new(1));
        assert_eq!(board.get(0, 254), NonZeroU16::new(254));
        assert_eq!(board.get(1, 0), NonZeroU16::new(255));
        assert_eq!(board.get(2, 0), NonZeroU16::new(510));
        assert_eq!(board.get(254, 254), NonZeroU16::new(65024));
        assert!(board.is_ordered());
    }
}
