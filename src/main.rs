mod rules;

use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rules::{Code, RULES};

const LOADING: [char; 6] = ['⠇', '⠋', '⠙', '⠸', '⠴', '⠦'];

type Feasible = (Vec<u8>, Code);
type Test = (usize, u8);

#[derive(Debug, Clone)]
struct Branch {
    test: Test,
    correct: BinaryTree,
    incorrect: BinaryTree,
    code: Option<Code>,
}

impl Branch {

    fn get_tests(&self, current_level: u8) -> HashSet<Test> {
        let mut tests = HashSet::new();
        tests.insert(self.test);
        if current_level % 3 != 2 {
            for c in self.correct.get_tests(current_level + 1) {
                tests.insert(c);
            }
            for i in self.incorrect.get_tests(current_level + 1) {
                tests.insert(i);
            }
        }
        tests
    }

    fn depth(&self) -> u8 {
        1 + self.correct.depth().max(self.incorrect.depth())
    }
}

#[derive(Debug, Clone)]
enum BinaryTree {
    Leaf(Code),
    Branch(Box<Branch>),
}

unsafe impl Send for BinaryTree {}
unsafe impl Sync for BinaryTree {}

impl BinaryTree {

    fn depth(&self) -> u8 {
        match self {
            BinaryTree::Leaf(_) => 0,
            BinaryTree::Branch(b) => b.depth(),
        }
    }

    fn size(&self) -> u8 {
        match self {
            BinaryTree::Leaf(_) => 1,
            BinaryTree::Branch(b) => 1 + b.correct.size() + b.incorrect.size(),
        }
    }

    fn get_tests(&self, current_level: u8) -> HashSet<Test> {
        match self {
            BinaryTree::Leaf(_) => HashSet::new(),
            BinaryTree::Branch(b) => b.get_tests(current_level),
        }
    }

    fn print(&self, indent: u8) {
        match self {
            BinaryTree::Leaf(c) => print!("{}\n", c.to_string()),
            BinaryTree::Branch(b) => {
                print!("Test: {:?}\n", b.test);
                for _ in 0..indent + 1 {
                    print!("  ");
                }
                print!("✓: ");
                b.correct.print(indent + 1);
                for _ in 0..indent + 1 {
                    print!("  ");
                }
                print!("✗: ");
                b.incorrect.print(indent + 1);
            },
        }
    }

}

#[derive(Debug)]
struct TestResult {
    test: Test,
    correct: Vec<Feasible>,
    incorrect: Vec<Feasible>,
}

impl TestResult {
    fn estimated_value(&self) -> usize { self.correct.len() * self.incorrect.len() }
}

fn sort_by_split(entries: &Vec<Feasible>, (i, v): Test) -> (Vec<Feasible>, Vec<Feasible>) {
    let mut correct: Vec<Feasible> = Vec::new();
    let mut incorrect: Vec<Feasible> = Vec::new();
    for e in entries {
        if e.0[i] == v {
            correct.push(e.clone());
        } else {
            incorrect.push(e.clone());
        }
    }
    (correct, incorrect)
}

fn get_permutations(input: &Vec<HashSet<u8>>) -> Vec<Vec<u8>> {
    let mut results = vec![Vec::new()];
    for i in 0..input.len() {
        let mut new_results = Vec::new();
        while let Some(r) = results.pop() {
            for v in &input[i] {
                let mut new = r.clone();
                new.push(*v);
                new_results.push(new);
            }
        }
        results = new_results;
    }
    results
}

fn construct_trees(entries: &Vec<Feasible>, tests: &Vec<HashSet<u8>>, sol_map: &HashMap<Vec<u8>,
    Vec<Code>>, current_level: u8, abort_level: Option<u8>, used_tests: &Vec<Test>) -> Vec<BinaryTree> {

    // identify leaves
    if entries.len() == 1 {
        return vec![BinaryTree::Leaf(entries[0].1.clone())];
    }

    // figure out possible tests.
    let mut nodes: Vec<TestResult> = Vec::new();
    tests.iter().enumerate().for_each(|(i, s)| {
        s.iter().for_each(|v| {
            let split = (i, *v);
            // ignore splits if the same test has been used in a previous attempt
            // this round
            if used_tests.iter().find(|(j, _)| *j == i).is_some() {
                return;
            }
            let (correct, incorrect) = sort_by_split(entries, split);
            if correct.is_empty() || incorrect.is_empty() {
                return;
            }
            nodes.push(TestResult { test: split, correct, incorrect });
        });
    });

    // heuristic: the more information we are guaranteed to get from a test,
    // the more promising it is.
    nodes.sort_by(|a, b| {
        b.estimated_value().cmp(&a.estimated_value())
    });

    // go through all possible tests and see what trees they yield.
    //let mut best_depth = None;
    let mut solutions = Vec::new();
    let mut best_depth = None;
    for node in nodes {

        // if we are in the middle of a round, make sure to mark
        // used tests for the next level.
        let next_splits = match current_level % 3 == 2 {
            true => Vec::new(),
            false => {
                let mut v = used_tests.clone();
                v.push(node.test.clone());
                v
            },
        };

        // within n levels, we can distinguish up to 2^n different solutions,
        // so we can abort if either solution is longer than that.
        let abort = match current_level == 0 {
            true => best_depth,
            false => abort_level,
        };
        if let Some(a) = abort {
            let max_splits = 1 << (a - 1 - current_level);
            if node.correct.len() > max_splits || node.incorrect.len() > max_splits {
                continue;
            }
        }

        // construct possible correct and incorrect subtrees
        let correct_trees = construct_trees(
            &node.correct,
            &tests,
            sol_map,
            current_level + 1,
            abort,
            &next_splits);
        let incorrect_trees = construct_trees(
            &node.incorrect,
            &tests,
            sol_map,
            current_level + 1,
            abort,
            &next_splits);

        // check the validity of each combination
        for correct_tree in &correct_trees {
            'outer: for incorrect_tree in &incorrect_trees {
                for (test_c, res_c) in correct_tree.get_tests(current_level) {
                    for (test_i, res_i) in incorrect_tree.get_tests(current_level) {
                        if test_c == test_i && res_c != res_i {
                            continue 'outer;
                        }
                    }
                }
                let mut branch = Branch {
                    test: node.test,
                    correct: correct_tree.clone(),
                    incorrect: incorrect_tree.clone(),
                    code: None,
                };
                if let Some(d) = best_depth {
                    if d < branch.depth() {
                        continue;
                    }
                }
                if current_level % 3 == 0 {
                    let mut results = tests.clone();
                    for (test, res) in branch.get_tests(current_level) {
                        let mut set = HashSet::new();
                        set.insert(res);
                        results[test] = set;
                    }
                    let permutations = get_permutations(&results);
                    //println!("{:?}", permutations);
                    let mut okay = false;
                    for p in &permutations {
                        if let Some(codes) = sol_map.get(p) {
                            branch.code = Some(codes[0].clone());
                            okay = true;
                            break;
                        }
                    }
                    if !okay {
                        continue 'outer;
                    }
                }
                let tree = BinaryTree::Branch(Box::new(branch));
                best_depth = Some(tree.depth());
                solutions.push(tree);
            }
        }
    }

    // return all possible trees
    solutions
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
    if rules.len() < 4 {
        println!("Not enough input rules, aborting.");
        return;
    }

    // create all possible 3-digit codes
    println!("Generating codes ...");
    let codes = (0..125).map(|i| {
        Code {
            blue: i % 5 + 1,
            yellow: (i / 5) % 5 + 1,
            purple: (i / 25) + 1,
        }
    });

    // check which results these codes yield after running the "program".
    println!("Looking for feasible solutions ...");
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

    // only unique solutions are interesting.
    let unique_solutions: Vec<Feasible> = solutions.iter().filter_map(|(k, v)| {
        if v.len() == 1 {
            return Some((k.clone(), v[0].clone()));
        }
        None
    }).collect();
    println!("Found {} feasible solutions.", unique_solutions.len());
    if unique_solutions.len() == 0 {
        println!("This puzzle does not appear to be solvable. Please double-check your inputs.");
        return;
    }
    if verbose {
        unique_solutions.iter().for_each(|s| println!("{:?}", s));
    }

    // check which test results appear within those solutions.
    let mut possible_results = vec![HashSet::<u8>::new(); rules.len()];
    for i in 0..rules.len() {
        for s in &unique_solutions {
            possible_results[i].insert(s.0[i]);
        }
    }

    // construct solution trees.
    println!("Constructing solution trees ...");
    let (sender, recieiver) = mpsc::channel();
    thread::spawn(move || {
        let mut trees = construct_trees(
            &unique_solutions, 
            &possible_results, 
            &solutions,
            0, 
            None,
            &Vec::new());
        trees.sort_by(|a, b| {
            match a.depth().cmp(&b.depth()) {
                Ordering::Less => Ordering::Less,
                Ordering::Equal => a.size().cmp(&b.size()),
                Ordering::Greater => Ordering::Greater,
            }
        });
        let _ = sender.send(trees[0].clone());
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
        if let Ok(t) = recieiver.try_recv() {
            tree = Some(t);
            break;
        }
    }
    let mut tree = &tree.unwrap();
    if verbose {
        tree.print(0);
    }
    println!("Done!");

    // guide the user through performing the input checks.
    let mut level = 0;
    let mut current_code = None;
    while let BinaryTree::Branch(b) = tree {
        if level % 3 == 0 {
            println!("------");
            println!("Start of round {}", level / 3 + 1);
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
        println!("Does test {} yield a check mark? (y/n)", c);
        loop {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            match input.chars().nth(0) {
                Some('y') => {
                    tree = &b.correct;
                    break;
                },
                Some('n') => {
                    tree = &b.incorrect;
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

