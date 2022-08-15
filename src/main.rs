extern crate pest;
#[macro_use]
extern crate pest_derive;

mod graph;
mod hive;

use std::io::Write;

use hive::Hive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "hive.pest"]
struct HiveParser;

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
            line
        };

        match HiveParser::parse(Rule::command, &input) {
            Ok(mut commands) => {
                let command = commands.next().unwrap();
                assert_eq!(command.as_rule(), Rule::command);
                for expr in command.into_inner() {
                    assert_eq!(expr.as_rule(), Rule::expr);
                    let expr = expr.into_inner().next().unwrap();
                    match expr.as_rule() {
                        Rule::action_seq => {
                            let snapshot = hive.undo.snapshot();
                            for action in expr.into_inner() {
                                match action.as_rule() {
                                    Rule::pipe => {
                                        let mut pipe = action.into_inner();
                                        let mut a = pipe.next().unwrap();
                                        while let Some(dir) = pipe.next() {
                                            let b = pipe.next().unwrap();
                                            match dir.as_rule() {
                                                Rule::to => hive.pipe(a.as_str(), b.as_str()),
                                                Rule::from => hive.pipe(b.as_str(), a.as_str()),
                                                _ => unreachable!(),
                                            }
                                            a = b;
                                        }
                                    }
                                    Rule::delete => {
                                        let delete = action.into_inner().next().unwrap();
                                        match delete.as_rule() {
                                            Rule::node => {
                                                hive.remove_node(delete.as_str());
                                            }
                                            Rule::edge => {
                                                hive.delete_edge(delete.as_str().parse().unwrap());
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            hive.undo.pile(snapshot);
                        }
                        Rule::history => {
                            let history = expr.into_inner().next().unwrap();
                            match history.as_rule() {
                                Rule::quit => {
                                    // Lets clear our hive to early catch asserts on tear down
                                    hive.clear();
                                    return;
                                }
                                Rule::clear => {
                                    hive.clear();
                                }
                                Rule::pile => {
                                    let n = history
                                        .into_inner()
                                        .next()
                                        .unwrap()
                                        .as_str()
                                        .parse()
                                        .unwrap();
                                    let snapshot = hive.undo.snapshot().saturating_sub(n);
                                    hive.undo.pile(snapshot);
                                }
                                rule @ (Rule::undo | Rule::redo) => {
                                    let n = history
                                        .into_inner()
                                        .map(|u| u.as_str().parse().unwrap())
                                        .next()
                                        .unwrap_or(1);
                                    match rule {
                                        Rule::undo => hive.undo(n),
                                        Rule::redo => hive.redo(n),
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                print!("{:?}", hive);
            }
            Err(error) => println!("{error}"),
        }
    }
}
