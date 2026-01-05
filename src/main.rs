use automata::automata::npda;

fn main() {
    let input = include_str!("../example.npda");

    let table = match npda::TransitionTable::load_table(input) {
        Ok((ok, logs)) => {
            for log in logs.displayable() {
                println!("{log}")
            }
            ok
        }
        Err(logs) => {
            for log in logs.displayable() {
                println!("{log}")
            }
            return;
        }
    };

    let input = "aababdsaab";
    println!("running on: '{input}'");
    let mut simulator = npda::Simulator::begin(input, table);
    loop {
        match simulator.step() {
            npda::SimulatorResult::Pending => {}
            npda::SimulatorResult::Reject => {
                println!("REJECTED");
                break;
            }
            npda::SimulatorResult::Accept(npda) => {
                println!("ACCEPT: {npda:?}");
                break;
            }
        }
    }
}
