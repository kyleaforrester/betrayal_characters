use rand::prelude::SliceRandom;
use std::cmp;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Individual {
    pub id: usize,
    pub fitness: i32,
    rank_score: i32,
    avg_score: i32,
    four_indexes: usize,
    totals_diff: usize,
    pub x: i32,
    pub y: i32,
    chars: HashMap<String, HashMap<String, (Vec<i32>, usize)>>,
}

impl Individual {
    pub fn new(x: i32, y: i32, id: usize, rankings: &HashMap<String, Vec<String>>) -> Individual {
        let mut chars = HashMap::new();
        for name in rankings.get("Might").unwrap().iter() {
            let mut stats: HashMap<String, (Vec<i32>, usize)> = HashMap::new();
            stats.insert("Might".to_string(), (vec![2, 2, 3, 4, 5, 6, 7, 8], 2));
            stats.insert("Speed".to_string(), (vec![2, 2, 3, 4, 5, 6, 7, 8], 3));
            stats.insert("Know".to_string(), (vec![2, 2, 3, 4, 5, 6, 7, 8], 3));
            stats.insert("Sanity".to_string(), (vec![2, 2, 3, 4, 5, 6, 7, 8], 3));

            chars.insert(name.to_string(), stats);
        }

        let mut ind = Individual {
            id: id,
            fitness: i32::MAX,
            rank_score: i32::MAX,
            avg_score: i32::MAX,
            four_indexes: 0,
            totals_diff: usize::MAX,
            x: x,
            y: y,
            chars: chars,
        };
        ind.mutate();
        ind.score(rankings);
        ind
    }

    pub fn breed(
        parent_a: &Individual,
        parent_b: &Individual,
        x: i32,
        y: i32,
        id: usize,
        rankings: &HashMap<String, Vec<String>>,
    ) -> Individual {
        let mut chars = HashMap::new();
        for name in parent_a.chars.keys() {
            let character = if rand::random() {
                parent_a.chars.get(name).unwrap().clone()
            } else {
                parent_b.chars.get(name).unwrap().clone()
            };

            chars.insert(name.to_string(), character);
        }

        let mut ind = Individual {
            id: id,
            fitness: i32::MAX,
            rank_score: i32::MAX,
            avg_score: i32::MAX,
            four_indexes: 0,
            totals_diff: usize::MAX,
            x: x,
            y: y,
            chars: chars,
        };
        ind.mutate();
        ind.score(rankings);
        ind
    }

    fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        for stats in self.chars.values_mut() {
            let mut total: i32 = stats.values().map(|x| x.0.iter().sum::<i32>()).sum();

            for tup in stats.values_mut() {
                // Mutate the list of numbers
                for i in 0..tup.0.len() {
                    // Only mutate 10% of integers
                    if rand::random::<f32>() > 0.1 {
                        continue;
                    }

                    let mut possibilities = Vec::new();

                    // Cannot change starting value
                    let cond = i != tup.1;

                    // Cannot increment to be larger than next number
                    let inc_1 = i == 7 || (i < 7 && tup.0[i] < tup.0[i + 1]);
                    // Cannot increment to more than 2 from previous number
                    let inc_2 = i == 0 || (i > 0 && tup.0[i] <= tup.0[i - 1] + 1);
                    // Total cannot be more than 155
                    let inc_3 = total < 155;
                    // Lowest number must be 1 to 4
                    let inc_4 = i > 0 || tup.0[i] < 4;
                    // Cannot be greater than 8
                    let inc_5 = tup.0[i] < 8;

                    // Cannot decrement to be less than previous number
                    let dec_1 = i == 0 || (i > 0 && tup.0[i] > tup.0[i - 1]);
                    // Cannot decrement to be more than 2 less than next number
                    let dec_2 = i == 7 || (i < 7 && tup.0[i] >= tup.0[i + 1] - 1);
                    // Total cannot be less than 145
                    let dec_3 = total > 145;
                    // Highest number must be 5 to 8
                    let dec_4 = i < 7 || tup.0[i] > 5;
                    // Cannot be less than 1
                    let dec_5 = tup.0[i] > 1;

                    possibilities.push(0);
                    // Can we increment
                    if cond && inc_1 && inc_2 && inc_3 && inc_4 && inc_5 {
                        possibilities.push(1);
                    }
                    // Can we decrement
                    if cond && dec_1 && dec_2 && dec_3 && dec_4 && dec_5 {
                        possibilities.push(-1);
                    }

                    let delta = possibilities.choose(&mut rng).unwrap();
                    tup.0[i] += delta;
                    total += delta;
                }
            }

            // Mutate the indexes.
            // Find a pair of attributes
            // See which pairs of [3, 4, 5] of each could work
            // Then pick one.
            // 20% chance of happening.
            if rand::random::<f32>() < 0.2 {
                let attrs: Vec<String> = ["Might", "Know", "Speed", "Sanity"]
                    .choose_multiple(&mut rng, 2)
                    .map(|x| x.to_string())
                    .collect();
                let mut possibilities = Vec::new();
                for i in -1i32..2 {
                    for j in -1i32..2 {
                        let mut new_stats = stats.clone();
                        if i >= 0 {
                            new_stats.get_mut(&attrs[0]).unwrap().1 += i as usize;
                        } else {
                            new_stats.get_mut(&attrs[0]).unwrap().1 -= i.abs() as usize;
                        }
                        if j >= 0 {
                            new_stats.get_mut(&attrs[1]).unwrap().1 += j as usize;
                        } else {
                            new_stats.get_mut(&attrs[1]).unwrap().1 -= j.abs() as usize;
                        }

                        if Individual::valid_indexes(new_stats) {
                            possibilities.push((i, j));
                        }
                    }
                }
                let delta = possibilities.choose(&mut rng).unwrap();

                if delta.0 >= 0 {
                    stats.get_mut(&attrs[0]).unwrap().1 += delta.0 as usize;
                } else {
                    stats.get_mut(&attrs[0]).unwrap().1 -= delta.0.abs() as usize;
                }

                if delta.1 >= 0 {
                    stats.get_mut(&attrs[1]).unwrap().1 += delta.1 as usize;
                } else {
                    stats.get_mut(&attrs[1]).unwrap().1 -= delta.1.abs() as usize;
                }
            }

            // Mutate starting values
            // Find a pair of starting values that can increment/decrement together
            // Only has a small chance of happening.  20% chance of checking for this.
            if rand::random::<f32>() < 0.2 {
                let attrs: Vec<String> = ["Might", "Know", "Speed", "Sanity"]
                    .choose_multiple(&mut rng, 2)
                    .map(|x| x.to_string())
                    .collect();
                let dec_tup = stats.get(&attrs[0]).unwrap();
                let inc_tup = stats.get(&attrs[1]).unwrap();

                let dec_idx = dec_tup.1;
                let inc_idx = inc_tup.1;

                let can_dec = dec_tup.0[dec_tup.1] > 2
                    // cant be less than previous number
                    && dec_tup.0[dec_tup.1] > dec_tup.0[dec_tup.1 - 1]
                    // cant be two less than next number
                    && dec_tup.0[dec_tup.1] >= dec_tup.0[dec_tup.1 + 1] - 1
                    // Speed, Sanity, Know must be at least 3
                    && (attrs[0].as_str() == "Might" || dec_tup.0[dec_tup.1] > 3);

                let can_inc = inc_tup.0[inc_tup.1] < 6
                    // cant be greater than next number
                    && inc_tup.0[inc_tup.1] < inc_tup.0[inc_tup.1 + 1]
                    // cant be two greater than previous number
                    && inc_tup.0[inc_tup.1] <= inc_tup.0[inc_tup.1 - 1] + 1;

                if can_dec && can_inc {
                    stats.get_mut(&attrs[0]).unwrap().0[dec_idx] -= 1;
                    stats.get_mut(&attrs[1]).unwrap().0[inc_idx] += 1;
                }
            }
        }
    }

    fn valid_indexes(stats: HashMap<String, (Vec<i32>, usize)>) -> bool {
        // Indexes are all >= 2 and <= 4
        if !stats.values().all(|x| x.1 >= 2 && x.1 <= 4) {
            return false;
        }

        // Starting values sum to 15
        if stats.values().map(|x| x.0[x.1]).sum::<i32>() != 15 {
            return false;
        }

        // Constitution is 10 or 11
        let constitution: usize = stats.values().map(|x| x.1).sum();
        if constitution < 10 || constitution > 11 {
            //println!("\tFail due to constitution{:?}", stats);
            return false;
        }

        // Starting values are at least either 2 or 3
        for stat in stats.iter() {
            match stat.0.as_str() {
                "Might" => {
                    if stat.1 .0[stat.1 .1] < 2 {
                        return false;
                    }
                }
                _ => {
                    if stat.1 .0[stat.1 .1] < 3 {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn score(&mut self, rankings: &HashMap<String, Vec<String>>) {
        let mut rank_score = 0;
        let mut names: Vec<String> = self.chars.keys().map(|x| x.to_string()).collect();
        for tup in rankings.iter() {
            // Sort in descending order based on the f32 output of the stat
            names.sort_by(|a, b| {
                Individual::attr_score(self, b, tup.0)
                    .partial_cmp(&Individual::attr_score(self, a, tup.0))
                    .unwrap()
            });

            for name in names.iter() {
                let true_idx = tup.1.iter().position(|x| x == name).unwrap();
                let my_idx = names.iter().position(|x| x == name).unwrap();
                rank_score += (true_idx as i32 - my_idx as i32).abs();
            }
        }

        // Keep average of Might to around 3
        // Keep average of other 3 traits to around 4
        let mut avg_score = 0;
        for attr in rankings.keys() {
            let my_sum: i32 = self
                .chars
                .values()
                .map(|x| {
                    let tup = x.get(attr).unwrap();
                    tup.0[tup.1]
                })
                .sum();
            let my_avg = my_sum as f32 / (self.chars.keys().count() as f32);
            if attr == "Might" {
                avg_score += ((my_avg - 3.25).abs() * 10.0) as i32;
            } else {
                avg_score += ((my_avg - 4.0).abs() * 10.0) as i32;
            }
        }

        // Make for more interesting diversity by:
        // 1) Increase  extremist 4 indexes, up to half the population
        // 2) Balance 10 and 11 index totals
        let four_indexes = self
            .chars
            .values()
            .map(|x| x.values().filter(|tup| tup.1 == 4).count())
            .sum::<usize>();
        let ten_totals = self
            .chars
            .values()
            .filter(|x| x.values().map(|tup| tup.1).sum::<usize>() == 10)
            .count();
        let eleven_totals = self.chars.len() - ten_totals;
        let totals_diff = cmp::max(ten_totals, eleven_totals) - cmp::min(ten_totals, eleven_totals);
        let diversity = totals_diff - cmp::min(four_indexes, self.chars.len() / 2);

        self.avg_score = avg_score;
        self.rank_score = rank_score;
        self.totals_diff = totals_diff;
        self.four_indexes = four_indexes;
        self.fitness = avg_score + rank_score + diversity as i32;
    }

    fn attr_score(&self, name: &str, attr: &str) -> f32 {
        let tup = self.chars.get(name).unwrap().get(attr).unwrap();

        let mut weighted_sum = tup.0[tup.1] as f32;
        let mut weight: f32 = 0.5;
        let mut offset = 1;
        while offset <= tup.1 {
            weighted_sum +=
                (tup.0[tup.1 - offset] + tup.0[cmp::min(tup.1 + offset, 7)]) as f32 * weight;
            weight *= 0.5;
            offset += 1;
        }

        weighted_sum
    }
}
