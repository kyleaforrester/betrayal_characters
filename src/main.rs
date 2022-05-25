use std::fs;
use std::io::{self, Read};
use std::collections::HashMap;

const FILE = "rankings.txt";

struct Population {
    pop: Vec<Individual>,
    x_axis: i32,
    y_axis: i32,
    window: i32,
    pop_size: usize,
    rankings: HashMap<String, Vec<String>>,
}

impl Population {
    fn new(rankings: HashMap<String, Vec<String>>) -> Population {
        let pop_size = 1000;

        let mut pop = Vec::new();
        for _i in 0..pop_size {
            pop.push(Individual::new(&rankings));
        }

        Population {
            pop: pop,
            x_axis: 1000,
            y_axis: 1000,
            window: 100,
            pop_size: pop_size,
            rankings: rankings,
        }
    }

    fn run(&mut self, gens: usize) {
        for i in 0..gens {
            println!("Starting generation {} of {}", i, gens);

            let (parent_a, parent_b) = self.select_parents();

            self.pop.push(Individual::new(&parent_a, &parent_b, &self.rankings));
            self.pop.push(Individual::new(&parent_a, &parent_b, &self.rankings));
        }
    }

    fn select_parents(&mut self) {

        let candidates = loop {
            let x = rand::thread_rng().gen_range(0..self.x_axis);
            let y = rand::thread_rng().gen_range(0..self.y_axis);

            // Allow the grid of individuals to be borderless.  The edges are connected to each
            // other.
            let window = self.window/2;
            let cand_idxs: Vec<usize> = self.pop.iter().filter(|i| (i.x >= x - window && i.x <= x + window)
                                               || (x + window > self.x_axis && i.x <= (x + window) % self.x_axis)
                                               || (x - window < 0 && i.x >= self.x_axis + x - window))
                .filter(|i| (i.y >= y - window && i.y <= y + window)
                        || (y + window > self.y_axis && i.y <= (y + window) % self.y_axis)
                        || (y - window < 0 && i.y >= self.y_axis + x - window))
                .map(|i| i.id).collect();



}

fn main() {

    let buffer = fs::read_to_string(FILE).expect(format!("Could not read from FILE {}", FILE));

    let mut rankings: HashMap<String, Vec<String>> = HashMap::new();

    for chunk in buffer.split("\n\n").filter(|x| x.len() > 0) {
        let mut attribute = String::new();
        let mut ranking = Vec::new();
        for line in chunk.split("\n").enumerate() {
            if line.0 == 0 {
                attribute = line.1.strip_suffix(":").unwrap().to_string();
            } else {
                ranking.push(line.1.to_string());
            }
        }
        rankings.insert(attribute, ranking);
    }

    // Validate 4 categories present
    assert!(rankings.keys().count == 4);
    assert!(['Might', 'Speed', 'Know', 'Sanity'].iter().all(|x| rankings.contains_key(x)));

    // Validate all characters present in each category
    let set = HashSet::from_iter(rankings.values().nth(0).unwrap().iter());
    assert!(rankings.values().map(|x| HashSet::from_iter(x.iter())).all(|x| x == set));

    let mut pop = Population::new(rankings);
    let mut choice = String:new();
    loop {
        let prompt = "\nChoose from the following options:\n\
            1) Generate new population\n\
            2) Load population from file\n\
            3) Save population to file\n\
            4) Save best individual to file\n\
            5) Run generations\n\
            Choice: ";

        print!(prompt);

        io::stdin().read_line(&mut choice).expect("Could not read from stdin!");

        match choice {
            "1" => pop = Population::new(),
            "2" => pop = load_population(),
            "3" => save_population(&pop),
            "4" => save_individual(&pop),
            "5" => run_generations(&mut pop),
            _ => println!("Not a valid answer! User responded: {}", choice),
        }
    }
}

fn run_generations(&mut pop: Vec<Individual>) {
    let mut response = String::new();
    
    print!("How many generations to spawn? ");

    io::stdin().read_line(&mut gens).expect("Could not read from stdin!");

    let gens: usize = match response.trim().parse() {
        Some(num) => num,
        None => {
            println!("Not a recognized response: {}", response);
            return
        },
    };

    pop.run(gens);
}
