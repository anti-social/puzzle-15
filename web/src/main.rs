use std::num::NonZeroU16;

use game::{Board, RandomShuffle, Move};

use gloo::events::EventListener;

use wasm_bindgen::{JsCast, UnwrapThrowExt};

use yew::prelude::*;

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
    let is_solved = {
        let board = board.clone();
        use_state(move || {
            board.borrow().is_solved()
        })
    };
    let moves = use_state(|| 0);

    {
        let board = board.clone();
        let rows = rows.clone();
        let is_solved = is_solved.clone();
        let moves = moves.clone();
        use_effect(move || {
            let document = gloo::utils::document();
            let listener = EventListener::new(&document, "keydown", move |event| {
                let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
                // log::warn!("Key pressed: {:?}", event.key());

                if *is_solved {
                    return;
                }

                let mv = match event.key().as_str() {
                    "ArrowLeft" => Some(Move::Left),
                    "ArrowRight" => Some(Move::Right),
                    "ArrowUp" => Some(Move::Up),
                    "ArrowDown" => Some(Move::Down),
                    _ => None,
                };
                if let Some(mv) = mv {
                    if board.borrow_mut().move_once(mv) {
                        rows.set(board.borrow().rows().iter().map(|row| row.to_vec()).collect());
                        is_solved.set(board.borrow().is_solved());
                        moves.set(*moves + 1);
                    }
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
                if *is_solved {
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
            <p style="font-size: 0.9em; color: dimgrey">
                { "Use arrow keys for control" }
            </p>
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
