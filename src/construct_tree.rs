use std::{collections::{HashSet, HashMap}, cmp::Ordering};

pub type Test = (usize, u8);
pub type Feasible<T> = (Vec<u8>, T);

#[derive(Debug, Clone)]
pub struct Branch<T> {
    pub test: Test,
    pub correct: BinaryTree<T>,
    pub incorrect: BinaryTree<T>,
    pub code: Option<T>,
}

impl<T> Branch<T> {

    fn get_tests(&self, sub_levels: u8) -> HashSet<Test> {
        let mut tests = HashSet::new();
        tests.insert(self.test);
        if sub_levels > 0 {
            for c in self.correct.get_tests(sub_levels - 1) {
                tests.insert(c);
            }
            for i in self.incorrect.get_tests(sub_levels - 1) {
                tests.insert(i);
            }
        }
        tests
    }

    fn max_depth(&self) -> u8 {
        1 + self.correct.max_depth().max(self.incorrect.max_depth())
    }

    fn total_depth(&self) -> usize {
        1 + self.correct.total_depth() + self.incorrect.total_depth()
    }
}

#[derive(Debug, Clone)]
pub enum BinaryTree<T> {
    Leaf(T),
    Branch(Box<Branch<T>>),
}

unsafe impl<T> Send for BinaryTree<T> {}
unsafe impl<T> Sync for BinaryTree<T> {}

impl<T> BinaryTree<T> {

    pub fn max_depth(&self) -> u8 {
        match self {
            BinaryTree::Leaf(_) => 0,
            BinaryTree::Branch(b) => b.max_depth(),
        }
    }

    pub fn total_depth(&self) -> usize {
        match self {
            BinaryTree::Leaf(_) => 0,
            BinaryTree::Branch(b) => b.total_depth(),
        }
    }

    pub fn size(&self) -> u8 {
        match self {
            BinaryTree::Leaf(_) => 1,
            BinaryTree::Branch(b) => 1 + b.correct.size() + b.incorrect.size(),
        }
    }

    pub fn get_tests(&self, sub_levels: u8) -> HashSet<Test> {
        match self {
            BinaryTree::Leaf(_) => HashSet::new(),
            BinaryTree::Branch(b) => b.get_tests(sub_levels),
        }
    }

}

impl<T: ToString> BinaryTree<T> {

    pub fn print(&self, indent: u8) {
        match self {
            BinaryTree::Leaf(c) => print!("{}\n", c.to_string()),
            BinaryTree::Branch(b) => {
                print!("Test: {:?}\n", b.test);
                for _ in 0..indent + 1 {
                    print!("  ");
                }
                print!("\x1b[32m✓\x1b[0m: ");
                b.correct.print(indent + 1);
                for _ in 0..indent + 1 {
                    print!("  ");
                }
                print!("\x1b[31m✗\x1b[0m: ");
                b.incorrect.print(indent + 1);
            },
        }
    }

}

#[derive(Debug)]
struct TestResult<T> {
    test: Test,
    correct: Vec<Feasible<T>>,
    incorrect: Vec<Feasible<T>>,
}

impl<T> TestResult<T> {
    fn estimated_value(&self) -> usize { self.correct.len() * self.incorrect.len() }
}

fn sort_by_split<T: Clone>(entries: &Vec<Feasible<T>>, (i, v): Test) -> (Vec<Feasible<T>>, Vec<Feasible<T>>) {
    let mut correct = Vec::new();
    let mut incorrect = Vec::new();
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

pub fn optimal_tree<T: Clone>(entries: &Vec<Feasible<T>>,
    solution_map: &HashMap<Vec<u8>, Vec<T>>,
    tests_per_round: u8) -> Option<BinaryTree<T>> {

    if entries.is_empty() {
        return None;
    }

    // check which test results appear within the unique solutions.
    let mut tests = vec![HashSet::<u8>::new(); entries[0].0.len()];
    for i in 0..entries[0].0.len() {
        for s in entries {
            tests[i].insert(s.0[i]);
        }
    }

    // what would be the ideal solution tree?
    let size = entries.len();
    let last_pot_2 = 1 << size.ilog2();
    let deep = (size - last_pot_2) * 2;
    let shallow = size - deep;
    let total_size = shallow * last_pot_2 + deep * last_pot_2 * 2;

    // recursively construct solution trees
    let mut trees = construct_trees_rec(
        entries,
        &tests,
        solution_map, 
        0,
        None, 
        tests_per_round, 
        total_size,
        &Vec::new()
    );
    if trees.is_empty() {
        return None;
    }

    // order trees by quality
    trees.sort_by(|a, b| {
        match b.max_depth().cmp(&a.max_depth()) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => b.total_depth().cmp(&a.total_depth()),
            Ordering::Greater => Ordering::Greater,
        }
    });
    trees.pop()

}

fn construct_trees_rec<T: Clone>(entries: &Vec<Feasible<T>>,
    tests: &Vec<HashSet<u8>>,
    solution_map: &HashMap<Vec<u8>, Vec<T>>,
    current_level: u8,
    abort_level: Option<u8>,
    tests_per_round: u8,
    optimal_depth: usize,
    used_tests: &Vec<Test>) -> Vec<BinaryTree<T>> {

    // identify leaves
    if entries.len() == 1 {
        return vec![BinaryTree::Leaf(entries[0].1.clone())];
    }

    // figure out possible tests.
    let mut nodes: Vec<TestResult<T>> = Vec::new();
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
        let next_splits = match current_level % tests_per_round == tests_per_round - 1 {
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
        let correct_trees = match current_level % tests_per_round == tests_per_round - 1 {
            false => construct_trees_rec(
                &node.correct,
                &tests,
                solution_map,
                current_level + 1,
                abort,
                tests_per_round,
                optimal_depth,
                &next_splits),
            true => match optimal_tree(&node.correct, solution_map, tests_per_round) {
                Some(r) => vec![r],
                None => Vec::new(),
            },
        };
        let incorrect_trees = match current_level % tests_per_round == tests_per_round - 1 {
            false => construct_trees_rec(
                &node.incorrect,
                &tests,
                solution_map,
                current_level + 1,
                abort,
                tests_per_round,
                optimal_depth,
                &next_splits),
            true => match optimal_tree(&node.incorrect, solution_map, tests_per_round) {
                Some(r) => vec![r],
                None => Vec::new(),
            },
        };

        // check the validity of each combination
        let sub_levels = tests_per_round - (current_level % tests_per_round) - 1;
        for correct_tree in &correct_trees {
            'outer: for incorrect_tree in &incorrect_trees {
                for (test_c, res_c) in correct_tree.get_tests(sub_levels) {
                    for (test_i, res_i) in incorrect_tree.get_tests(sub_levels) {
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
                    if d < branch.max_depth() {
                        continue;
                    }
                }
                if current_level % tests_per_round == 0 {
                    let mut results = tests.clone();
                    for (test, res) in branch.get_tests(tests_per_round - 1) {
                        let mut set = HashSet::new();
                        set.insert(res);
                        results[test] = set;
                    }
                    let permutations = get_permutations(&results);
                    //println!("{:?}", permutations);
                    let mut okay = false;
                    for p in &permutations {
                        if let Some(codes) = solution_map.get(p) {
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
                best_depth = Some(tree.max_depth());
                let total_depth = tree.total_depth();
                solutions.push(tree);

                // let's be greedy: if we've found an optimal tree, we don't
                // have to keep looking for more.
                if current_level == 0 && total_depth == optimal_depth {
                    return solutions;
                }
            }
        }
    }

    // return all possible trees
    solutions
}