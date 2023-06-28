use std::num::NonZeroU16;

use game::{Board, RandomShuffle, Move};

use gloo::events::EventListener;

use wasm_bindgen::{JsCast, UnwrapThrowExt};

use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let rng = rand::thread_rng();
    let shuffle = use_mut_ref(|| RandomShuffle::new(rng));
    let board = use_mut_ref(|| Board::new(4, &mut *shuffle.borrow_mut()));
    let moves = use_state(|| 0);

    {
        let board = board.clone();
        let moves = moves.clone();
        use_effect(move || {
            let document = gloo::utils::document();
            let listener = EventListener::new(&document, "keydown", move |event| {
                let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
                // log::warn!("Key pressed: {:?}", event.key());

                let mut board = board.borrow_mut();
                if board.is_solved() {
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
                    if board.move_once(mv) {
                        moves.set(*moves + 1);
                    }
                }
            });

            // Called when the component is unmounted.  The closure has to hold on to `listener`, because if it gets
            // dropped, `gloo` detaches it from the DOM. So it's important to do _something_, even if it's just dropping it.
            || drop(listener)
        });
    }

    let restart_game = {
        let board = board.clone();
        let shuffle = shuffle.clone();
        let moves = moves.clone();
        Callback::from(
            move |_| {
                board.borrow_mut().reset(&mut *shuffle.borrow_mut());
                moves.set(0);
            }
        )
    };

    {
        let board = board.borrow();
        html! {
            <div style="width: 400px; margin: auto">
                <h1>
                    { "Puzzle 15 game" }
                </h1>
                <h2>
                    if board.is_solved() {
                        { format!("Puzzle solved for {} moves", *moves) }
                    } else {
                        { format!("{} moves", *moves) }
                    }
                </h2>
                <div style="width: 400px; height: 400px; font-size: 40pt">
                    <div style="display: grid; grid-template-columns: repeat(4, 1fr); grid-gap: 5px">
                        {
                            board.rows().iter()
                                .map(|row| html! {
                                    <GameBoardRow row={ row.to_vec() }/>
                                })
                                .collect::<Html>()
                        }
                    </div>
                </div>
                <div style="display: grid; grid-template-columns: 3fr 1fr">
                    <p style="font-size: 0.9em; color: dimgrey">
                        { "Use arrow keys for control" }
                    </p>
                    <button onclick={ restart_game }>
                        { "New game" }
                    </button>
                </div>
            </div>
        }
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
