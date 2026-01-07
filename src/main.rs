use automata::{automata::npda, loader::Context};

fn main() {
    let input = include_str!("../example.npda");
    let mut ctx = Context::new(input);

    let machine = automata::loader::parse_universal(&mut ctx);
    for log in ctx.logs_display(){
        println!("{log}")
    }
    
    let machine = match machine{
        Some(automata::loader::Machine::Npda(npda)) => {
            npda
        },
        None => return,
    };

    let input = "aababaaba";
    println!("running on: '{input}'");
    let mut simulator = npda::Simulator::begin(input, machine);
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
