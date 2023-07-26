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
    let cur_touch = use_state(|| None);

    {
        let board = board.clone();
        let moves = moves.clone();
        let cur_touch_end = cur_touch.clone();

        use_effect(move || {
            let document = gloo::utils::document();

            let keyboard_listener = {
                let board = board.clone();
                let moves = moves.clone();

                EventListener::new(&document, "keydown", move |event| {
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
                })
            };

            let touch_start_listener = {
                let board = board.clone();

                EventListener::new(&document, "touchstart", move |event| {
                    let board = board.borrow();
                    if board.is_solved() {
                        return;
                    }

                    let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
                    if let Some(touch) = event.changed_touches().get(0) {
                        let (id, x, y) = (touch.identifier(), touch.screen_x(), touch.screen_y());
                        cur_touch.set(Some((id, x, y)));
                    }
                })
            };

            let touch_end_listener = EventListener::new(&document, "touchend", move |event| {
                let mut board = board.borrow_mut();
                if board.is_solved() {
                    return;
                }

                let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
                if let Some(touch) = event.changed_touches().get(0) {
                    let (id, x, y) = (touch.identifier(), touch.screen_x(), touch.screen_y());

                    if let Some((start_id, start_x, start_y)) = *cur_touch_end {
                        if id != start_id {
                            return;
                        }
                        let diff_x = x - start_x;
                        let diff_y = y - start_y;
                        // log::warn!("diff_x: {diff_x}, diff_y: {diff_y}");

                        let maybe_move = if diff_x.abs() > diff_y.abs() {
                            if diff_x > 0 {
                                Some(Move::Right)
                            } else if diff_x < 0 {
                                Some(Move::Left)
                            } else {
                                None
                            }
                        } else if diff_x.abs() < diff_y.abs() {
                            if diff_y > 0 {
                                Some(Move::Down)
                            } else if diff_y < 0 {
                                Some(Move::Up)
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        if let Some(mv) = maybe_move {
                            if board.move_once(mv) {
                                moves.set(*moves + 1);
                            }
                        }
                        cur_touch_end.set(None);
                    }
                }
            });


            // Called when the component is unmounted.  The closure has to hold on to `listener`, because if it gets
            // dropped, `gloo` detaches it from the DOM. So it's important to do _something_, even if it's just dropping it.
            || {
                drop(keyboard_listener);
                drop(touch_start_listener);
                drop(touch_end_listener);
            }
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
