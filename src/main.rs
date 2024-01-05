mod rules;
mod construct_tree;

use std::collections::HashMap;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rules::{Code, RULES};
use construct_tree::{BinaryTree, Feasible};

const CHECKMARK: &'static str = "\x1b[32m✓\x1b[0m";
const LOADING: [char; 6] = ['⠇', '⠋', '⠙', '⠸', '⠴', '⠦'];

fn do_task<F: Send + 'static + FnOnce() -> T, T: Send + 'static>(message: &str, task: F) -> T {
    print!("{} ", message);
    let mut i = 0;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let res = task();
        let _ = sender.send(res);
    });
    loop {
        print!("{} ", LOADING[i]);
        let _ = std::io::stdout().flush();
        i = (i + 1) % 6;
        thread::sleep(Duration::from_millis(100));
        print!("\x08\x08");
        if let Ok(t) = receiver.try_recv() {
            print!("{} \n", CHECKMARK);
            return t;
        }
    }
}

fn main() {

    // what rules are used?
    let input = std::env::args();
    let mut verbose = false;
    let rules: Vec<usize> = input.filter_map(|l| {
        if l == "-v" {
            verbose = true;
        }
        let rule = l.parse::<usize>();
        if let Ok(r) = rule {
            if r > 0 && r <= RULES.len() {
                return Some(r - 1);
            }
        }
        //println!("{} could not be parsed as a valid rule number.", l);
        None
    }).collect();
    let no_rules = rules.len();
    if no_rules < 4 {
        println!("Not enough input rules, aborting.");
        return;
    }

    // create all possible 3-digit codes
    let codes = do_task("Generating codes ...", || (0..125).map(|i| {
        Code {
            blue: i % 5 + 1,
            yellow: (i / 5) % 5 + 1,
            purple: (i / 25) + 1,
        }
    }));

    // check which results these codes yield after running the "program".
    let solutions = do_task("Looking for unique solutions ...", move || {
        let mut solutions: HashMap<Vec<u8>, Vec<Code>> = HashMap::new();
        codes.for_each(|code| {
            let results: Vec<u8> = rules.iter().filter_map(|rule| {
                RULES[*rule](&code)
            }).collect();
            if results.len() < rules.len() {
                return;
            }
            match solutions.get_mut(&results) {
                Some(cur) => cur.push(code),
                None => {
                    solutions.insert(results, vec![code]);
                },
            }
        });
        solutions
    });

    // only unique solutions are interesting.
    let unique_solutions: Vec<Feasible<Code>> = solutions.iter().filter_map(|(k, v)| {
        if v.len() == 1 {
            return Some((k.clone(), v[0].clone()));
        }
        None
    }).collect();
    if unique_solutions.len() == 0 {
        println!("This puzzle does not appear to be solvable. Please double-check your inputs.");
        return;
    }
    if verbose {
        for i in 0..no_rules {
            print!(" {} ", (i as u8 + 0x41) as char)
        }
        print!("\n");
        unique_solutions.iter().for_each(|s| println!("{:?} -> {}", s.0, s.1.to_string()));
    }

    let tree = do_task("Construct optimal tree ...", move || {
        construct_tree::optimal_tree(&unique_solutions, &solutions, 3)
    });
    let mut tree = tree.unwrap();
    if verbose {
        tree.print(0);
    }

    // construct an optimal solution tree
   /*  println!("Construct optimal tree ...");
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let tree = construct_tree::optimal_tree(&unique_solutions, &solutions, 3);
        let _ = sender.send(tree.unwrap());
    });

    // just a cute little loading indicator while we wait for the main thread
    // to finish.
    let mut load_i = 0;
    let mut tree = None;
    loop {
        print!("{} ", LOADING[load_i]);
        let _ = std::io::stdout().flush();
        load_i = (load_i + 1) % 6;
        thread::sleep(Duration::from_millis(100));
        print!("\x08\x08");
        if let Ok(t) = receiver.try_recv() {
            tree = Some(t);
            break;
        }
    }
    let mut tree = &tree.unwrap();
    if verbose {
        tree.print(0);
    }
    println!("Done!"); */

    // guide the user through performing the input checks.
    let mut level = 0;
    let mut current_code = None;
    while let BinaryTree::Branch(b) = tree {
        if level % 3 == 0 {
            println!("------");
            println!("\x1b[1mStart of round {}\x1b[0m", level / 3 + 1);
            current_code = b.code.clone();
            println!("Use the following combination: {}", current_code.unwrap().to_string());
        }
        let c = match b.test.0 {
            0 => 'A',
            1 => 'B',
            2 => 'C',
            3 => 'D',
            4 => 'E',
            5 => 'F',
            _ => '?',
        };
        println!("Does \x1b[47m Test {} \x1b[0m yield a {} ? (y/n)", c, CHECKMARK);
        loop {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            match input.chars().nth(0) {
                Some('y') => {
                    tree = b.correct;
                    break;
                },
                Some('n') => {
                    tree = b.incorrect;
                    break;
                },
                _ => println!("Please input y or n."),
            }
        }
        level += 1;
    }

    // done!
    if let BinaryTree::Leaf(c) = tree {
        println!("Found a solution!");
        println!("Your code is: {}", c.to_string());
    } else {
        println!("Something went terribly wrong and I don't know what it is. Sorry!");
    }

}

