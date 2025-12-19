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
}
