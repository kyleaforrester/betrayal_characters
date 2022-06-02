use individual::Individual;
use rand::prelude::SliceRandom;
use rand::Rng;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Write};

mod individual;

const FILE: &str = "rankings.txt";

struct Population {
    pop: Vec<Individual>,
    x_axis: i32,
    y_axis: i32,
    window: i32,
    pop_size: usize,
    rankings: HashMap<String, Vec<String>>,
    counter: usize,
}

impl Population {
    fn new(rankings: HashMap<String, Vec<String>>) -> Population {
        let mut rng = rand::thread_rng();
        let pop_size = 1000;
        let mut counter = 0;
        let x_axis = 1000;
        let y_axis = 1000;

        let mut pop = Vec::new();
        for _i in 0..pop_size {
            let new_x = rng.gen_range(0..x_axis);
            let new_y = rng.gen_range(0..y_axis);
            pop.push(Individual::new(new_x, new_y, counter, &rankings));
            counter += 1;
        }

        Population {
            pop: pop,
            x_axis: x_axis,
            y_axis: y_axis,
            window: 100,
            pop_size: pop_size,
            rankings: rankings,
            counter: counter,
        }
    }

    fn run(&mut self, gens: usize) {
        for i in 0..gens {
            println!("Starting generation {} of {}", i + 1, gens);

            let mut rng = &mut rand::thread_rng();
            loop {
                let x = rng.gen_range(0..self.x_axis);
                let y = rng.gen_range(0..self.y_axis);

                // Allow the grid of individuals to be borderless.  The edges are connected to each
                // other.
                let h_w = self.window / 2;
                let mut cands: Vec<Individual> = self
                    .pop
                    .iter()
                    .filter(|i| {
                        (i.x >= x - h_w && i.x <= x + h_w)
                            || (x + h_w > self.x_axis && i.x <= (x + h_w) % self.x_axis)
                            || (x - h_w < 0 && i.x >= self.x_axis + x - h_w)
                    })
                    .filter(|i| {
                        (i.y >= y - h_w && i.y <= y + h_w)
                            || (y + h_w > self.y_axis && i.y <= (y + h_w) % self.y_axis)
                            || (y - h_w < 0 && i.y >= self.y_axis + y - h_w)
                    })
                    .cloned()
                    .collect();

                if cands.len() < 4 {
                    continue;
                }

                // Randomly select 4 candidates
                cands = cands.choose_multiple(&mut rng, 4).cloned().collect();

                // Sort by fitness ascending
                cands.sort_by_key(|x| x.fitness);

                // Kill the two worst solutions
                self.cull(cands.pop().unwrap().id);
                self.cull(cands.pop().unwrap().id);

                // Breed the two best
                for _i in 0..2 {
                    let new_x = rng.gen_range(
                        cmp::max(0, self.x_axis - h_w)..cmp::min(self.x_axis + h_w, self.x_axis),
                    );
                    let new_y = rng.gen_range(
                        cmp::max(0, self.y_axis - h_w)..cmp::min(self.y_axis + h_w, self.y_axis),
                    );
                    self.pop.push(Individual::breed(
                        &cands[0],
                        &cands[1],
                        new_x,
                        new_y,
                        self.counter,
                        &self.rankings,
                    ));
                    self.counter += 1;
                }
                break;
            }
        }
    }

    fn cull(&mut self, id: usize) {
        self.pop.remove(
            self.pop
                .iter()
                .enumerate()
                .find(|x| x.1.id == id)
                .unwrap()
                .0,
        );
    }
}

fn main() {
    let buffer =
        fs::read_to_string(FILE).expect(format!("Could not read from FILE {}", FILE).as_str());

    let mut rankings: HashMap<String, Vec<String>> = HashMap::new();

    for chunk in buffer.split("\n\n").filter(|x| x.len() > 0) {
        let mut attribute = String::new();
        let mut ranking = Vec::new();
        for line in chunk.split("\n").filter(|x| x.len() > 0).enumerate() {
            if line.0 == 0 {
                attribute = line.1.strip_suffix(":").unwrap().to_string();
            } else {
                ranking.push(line.1.to_string());
            }
        }
        rankings.insert(attribute, ranking);
    }

    // Validate 4 categories present
    assert!(rankings.keys().count() == 4);
    assert!(["Might", "Speed", "Know", "Sanity"]
        .iter()
        .all(|x| rankings.contains_key(*x)));

    // Validate all characters present in each category
    let set = HashSet::from_iter(rankings.values().nth(0).unwrap().iter());
    for v in rankings.values() {
        println!("{:?}", v);
    }
    assert!(rankings
        .values()
        .map(|x| HashSet::from_iter(x.iter()))
        .all(|x: HashSet<&String>| x == set));

    let mut pop = Population::new(rankings.clone());
    loop {
        let mut choice = String::new();
        let prompt = "\nChoose from the following options:\n\
            1) Generate new population\n\
            2) Load population from file\n\
            3) Save population to file\n\
            4) Save best individual to file\n\
            5) Run generations\n\
            6) Print best individual\n\
            7) Print population\n
            Choice: ";

        print!("{}", prompt);

        io::stdin()
            .read_line(&mut choice)
            .expect("Could not read from stdin!");

        match choice.trim() {
            "1" => pop = Population::new(rankings.clone()),
            "2" => pop = load_population(&rankings),
            "3" => save_population(&pop),
            "4" => save_individual(&pop),
            "5" => run_generations(&mut pop),
            "6" => print_individual(&pop),
            "7" => print_population(&pop),
            _ => println!("Not a valid answer! User responded: {}", choice),
        }
    }
}

fn run_generations(pop: &mut Population) {
    let mut response = String::new();

    print!("How many generations to spawn? ");
    io::stdout().flush().unwrap();

    io::stdin()
        .read_line(&mut response)
        .expect("Could not read from stdin!");

    let gens: usize = match response.trim().parse() {
        Ok(num) => num,
        Err(e) => {
            println!("Not a recognized response: {}. Error: {}", response, e);
            return;
        }
    };

    pop.run(gens);
}

fn load_population(rankings: &HashMap<String, Vec<String>>) -> Population {
    Population::new(rankings.clone())
}

fn save_population(_pop: &Population) {
    ()
}

fn save_individual(_pop: &Population) {
    ()
}

fn print_individual(pop: &Population) {
    println!("{:?}", pop.pop.iter().min_by_key(|x| x.fitness));
}

fn print_population(pop: &Population) {
    for i in pop.pop.iter() {
        println!("{:?}", i);
    }
}
