use std::marker::PhantomData;
use std::num::NonZeroU16;

use game::{Board, Move, Shuffle};

use gloo::events::EventListener;

use rand::prelude::*;

use wasm_bindgen::{JsCast, UnwrapThrowExt};

use yew::prelude::*;

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

#[function_component]
fn App() -> Html {
    let rng = rand::thread_rng();
    let mut shuffle = RandomShuffle::new(rng);
    let board = use_mut_ref(|| Board::new(4, &mut shuffle).unwrap());
    let rows = {
        let board = board.clone();
        use_state(move || {
            board.borrow().rows()
                .iter()
                .map(|row| row.to_vec())
                .collect::<Vec<_>>()
        })
    };
    let is_ordered = {
        let board = board.clone();
        use_state(move || {
            board.borrow().is_ordered()
        })
    };
    let moves = use_state(|| 0);

    {
        let board = board.clone();
        let rows = rows.clone();
        let is_ordered = is_ordered.clone();
        let moves = moves.clone();
        use_effect(move || {
            let document = gloo::utils::document();
            let listener = EventListener::new(&document, "keydown", move |event| {
                let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
                // log::warn!("Key pressed: {:?}", event.key());

                let mv = match event.key().as_str() {
                    "ArrowLeft" => Some(Move::Left),
                    "ArrowRight" => Some(Move::Right),
                    "ArrowUp" => Some(Move::Up),
                    "ArrowDown" => Some(Move::Down),
                    _ => None,
                };
                if let Some(mv) = mv {
                    board.borrow_mut().move_once(mv);
                    rows.set(board.borrow().rows().iter().map(|row| row.to_vec()).collect());
                    is_ordered.set(board.borrow().is_ordered());
                    moves.set(*moves + 1)
                }
            });

            // Called when the component is unmounted.  The closure has to hold on to `listener`, because if it gets
            // dropped, `gloo` detaches it from the DOM. So it's important to do _something_, even if it's just dropping it.
            || drop(listener)
        });
    }

    html! {
        <div style="width: 400px; margin: auto">
            <h1>
                { "Puzzle 15 game" }
            </h1>
            <h2>
                if *is_ordered {
                    { format!("Puzzle solved for {} moves", *moves) }
                } else {
                    { format!("{} moves", *moves) }
                }
            </h2>
            <div style="width: 400px; height: 400px; font-size: 40pt">
                <div style="display: grid; grid-template-columns: repeat(4, 1fr); grid-gap: 5px">
                    {
                        rows.iter()
                            .map(|row| html! {
                                <GameBoardRow row={ row.to_vec() }/>
                            })
                            .collect::<Html>()
                    }
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct GameBoardRowProps {
    row: Vec<Option<NonZeroU16>>,
}

#[function_component]
fn GameBoardRow(props: &GameBoardRowProps) -> Html {
    let row = &props.row;

    row.iter()
       .map(|cell| html! {
           <div style="width: 90px; height: 90px; text-align: center; border: 1px solid orange">
               { format!("{}", cell.map_or("".to_string(), |v| v.to_string())) }
           </div>
       })
       .collect()
}

fn main() {
    wasm_logger::init(Default::default());
    yew::Renderer::<App>::new().render();
}
