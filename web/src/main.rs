use automata::automata::npda;

use web_sys::window;

fn main() {
    console_error_panic_hook::set_once();

    let document = window()
        .and_then(|win| win.document())
        .expect("Could not access the document");
    let body = document.body().expect("Could not access document.body");
    let text_node = document.create_text_node("Hello, world from Vanilla Rust!");
    body.append_child(text_node.as_ref())
        .expect("Failed to append text");
}

// pub fn main() {
//     let input = include_str!("../../example.npda");

//     let table = match npda::TransitionTable::load_table(input) {
//         Ok((ok, logs)) => {
//             for log in logs.displayable() {
//                 println!("{log}")
//             }
//             ok
//         }
//         Err(logs) => {
//             for log in logs.displayable() {
//                 println!("{log}")
//             }
//             return;
//         }
//     };

//     let input = "aababaab";
//     println!("running on: '{input}'");
//     let mut simulator = npda::Simulator::begin(input, table);
//     loop {
//         match simulator.step(){
//             npda::SimulatorResult::Pending => {},
//             npda::SimulatorResult::Reject => {
//                 println!("REJECTED");
//                 break;
//             },
//             npda::SimulatorResult::Accept(npda) => {
//                 println!("ACCEPT: {npda:?}");
//                 break;
//             },
//         }
//     }
// }
