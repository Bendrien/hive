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
        let mut ignored = Vec::new();

        let mut args = &args[..];
        loop {
            args = match *hive.parse(args) {
                [] => {
                    if !ignored.is_empty() {
                        println!("Ignored input {:?}", ignored);
                        ignored.clear();
                    }
                    print!("{:?}", hive);
                    break;
                }
                ["q" | "quit", ..] => return,
                [ref xs @ ..] if xs.len() != args.len() => xs,
                [ignore, ref xs @ ..] => {
                    ignored.push(ignore);
                    xs
                }
            }
        }
    }
}
