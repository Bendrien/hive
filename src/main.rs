mod graph;
mod hive;

use std::io::Write;

use hive::Hive;

fn main() {
    let mut hive = Hive::default();
    loop {
        let input = {
            print!("» ");
            let mut line = String::new();
            std::io::stdout().flush().unwrap();
            std::io::stdin()
                .read_line(&mut line)
                .expect("Error: Could not read a line");
            line.trim().to_string()
        };

        let args = input.split(' ').collect::<Vec<_>>();
        let mut args = &args[..];
        let mut ignored = Vec::new();

        let mut snapshot = hive.undo.snapshot();
        loop {
            args = match *hive.parse(args) {
                [] => {
                    if !ignored.is_empty() {
                        println!("Ignored input {:?}", ignored);
                        ignored.clear();
                    }
                    hive.undo.pile(snapshot);
                    print!("{:?}", hive);
                    break;
                }
                [";", ref xs @ ..] => {
                    hive.undo.pile(snapshot);
                    snapshot = hive.undo.snapshot();
                    xs
                }
                ["q" | "quit", ..] => {
                    // Lets clear our hive to early catch asserts on tear down
                    hive.clear();
                    return;
                }
                [ref xs @ ..] if xs.len() != args.len() => xs,
                [ignore, ref xs @ ..] => {
                    ignored.push(ignore);
                    xs
                }
            }
        }
    }
}
