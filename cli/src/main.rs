use std::io::{self, BufRead, Write};
use std::marker::PhantomData;

use clap::{arg, command, Parser};

use game::{Board, BoardShuffle, DummyShuffle, Move, Shuffle};

use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::rngs::ThreadRng;

fn display_board(output: &mut impl Write, board: &Board) -> anyhow::Result<()> {
    for row in board.rows() {
        for cell in row {
            if let Some(cell_val) = cell {
                write!(output, "{cell_val:4}")?;
            } else {
                write!(output, "    ")?;
            }
        }
        writeln!(output)?;
        writeln!(output)?;
    }

    Ok(())
}

enum Cmd {
    Moves(Vec<Move>),
    Quit,
}

fn parse_cmd(s: &str) -> Cmd {
    let mut moves = vec!();
    for c in s.chars() {
        let mv = match c {
            'w' => Move::Up,
            'a' => Move::Left,
            's' => Move::Down,
            'd' => Move::Right,
            'q' => return Cmd::Quit,
            _ => {
                // Just ignore unknown directions
                continue;
            }
        };
        moves.push(mv);
    }
    Cmd::Moves(moves)
}

fn run(mut input: impl BufRead, mut output: impl Write, shuffle: &mut BoardShuffle) -> anyhow::Result<()> {
    let mut board = Board::new(4, shuffle)?;
    display_board(&mut output, &board)?;

    let mut input_buf = String::new();
    loop {
        write!(output, "Slide into direction [w, a, s, d], q - for quit: ")?;
        output.flush()?;
        input.read_line(&mut input_buf)?;

        match parse_cmd(&input_buf) {
            Cmd::Moves(moves) => {
                board.move_many(&moves);
            }
            Cmd::Quit => return Ok(()),
        }
        display_board(&mut output, &board)?;
        if board.is_ordered() {
            writeln!(output, "Puzzle is solved!\n")?;
        }
        input_buf.clear();
    }
}

struct RandomShuffle<T> {
    rng: ThreadRng,
    _marker: PhantomData<T>,
}

impl<T> RandomShuffle<T> {
    fn new(rng: ThreadRng) -> Self {
        Self { rng, _marker: PhantomData::default() }
    }
}

impl<T> Shuffle for RandomShuffle<T> {
    type Item = T;

    fn shuffle(&mut self, data: &mut Vec<Self::Item>) {
        data.shuffle(&mut self.rng)
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    no_shuffle: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut shuffle: Box<BoardShuffle> = if args.no_shuffle {
        Box::new(DummyShuffle::default())
    } else {
        let rng = thread_rng();
        Box::new(RandomShuffle::new(rng))
    };
    let input = io::stdin().lock();
    run(input, io::stdout(), shuffle.as_mut())
}

#[cfg(test)]
mod tests {
    use game::{Board, DummyShuffle};

    use super::{display_board, run};

    #[test]
    fn test_display_board() -> anyhow::Result<()> {
        let mut output = vec!();

        let board = Board::new(4, &mut DummyShuffle::default())?;
        display_board(&mut output, &board)?;

        similar_asserts::assert_eq!(
            String::from_utf8(output)?,
            "       1   2   3\n\n   \
            4   5   6   7\n\n   \
            8   9  10  11\n\n  \
            12  13  14  15\n\n"
        );

        Ok(())
    }

    #[test]
    fn test_run() -> anyhow::Result<()> {
        let input = b"aaw\nq\n";
        let mut output = vec!();

        run(&input[..], &mut output, &mut DummyShuffle::default())?;

        similar_asserts::assert_eq!(
            String::from_utf8(output)?,
            "       1   2   3\n\n   \
            4   5   6   7\n\n   \
            8   9  10  11\n\n  \
            12  13  14  15\n\n\
            Slide into direction [w, a, s, d], q - for quit:    \
            1   2   6   3\n\n   \
            4   5       7\n\n   \
            8   9  10  11\n\n  \
            12  13  14  15\n\n\
            Slide into direction [w, a, s, d], q - for quit: "
        );

        Ok(())
    }

    #[test]
    fn test_run_solved() -> anyhow::Result<()> {
        let input = b"ad\nq\n";
        let mut output = vec!();

        run(&input[..], &mut output, &mut DummyShuffle::default())?;

        similar_asserts::assert_eq!(
            String::from_utf8(output)?,
            "       1   2   3\n\n   \
            4   5   6   7\n\n   \
            8   9  10  11\n\n  \
            12  13  14  15\n\n\
            Slide into direction [w, a, s, d], q - for quit:        \
            1   2   3\n\n   \
            4   5   6   7\n\n   \
            8   9  10  11\n\n  \
            12  13  14  15\n\n\
            Puzzle is solved!\n\n\
            Slide into direction [w, a, s, d], q - for quit: "
        );

        Ok(())
    }
}
