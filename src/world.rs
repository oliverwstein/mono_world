use rand::Rng;
use std::collections::{HashMap, HashSet};
pub struct World {
    pub day: usize,
    pub entities: usize,
    pub world_height: usize,
    pub world_width: usize,
    pub positions: Vec<usize>,
    pub lives: Vec<bool>,
    pub spawn_dates: Vec<usize>,
    pub males: Vec<bool>,
    pub females: Vec<bool>,
    pub humans: Vec<bool>,
    pub forages: Vec<u8>,
    pub residents: HashMap<usize, HashSet<usize>>,
    pub mates: Vec<usize>,
    pub parents: HashMap<usize, Vec<usize>>,
    pub children: HashMap<usize, HashSet<usize>>,
    pub fertile: Vec<bool>,
    pub pregnant: Vec<usize>,
}


impl World {
    pub fn new(world_height: usize, world_width: usize) -> Self {
        let grid_size = world_height * world_width + 1;
        let mut forages = Vec::with_capacity(grid_size);
        for _i in 1..grid_size {
            let bounty = rand::thread_rng().gen_range(1..=10);
            forages.push(bounty);
        }

        World {
            entities: 0,
            day: 36500,
            world_height,
            world_width,
            positions: vec![0; 65535],
            lives: vec![false; 65535],
            spawn_dates: vec![0; 65535],
            males: vec![false; 65535],
            females: vec![false; 65535],
            humans: vec![false; 65535],
            forages,
            residents: HashMap::new(),
            mates: vec![0; 65535],
            fertile: vec![false; 65535],
            pregnant:vec![0; 65535],
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
    
    pub fn create_entity(&mut self) -> usize {
        self.entities += 1;
        self.entities
    }

    pub fn die(&mut self, index: usize) {
        // Remove the entity from its current position's residents set
        if let Some(residents) = self.residents.get_mut(&self.positions[index]) {
            residents.remove(&index);
        }
        // Reset the entity's position and status flags
        self.positions[index] = 0;
        self.lives[index] = false;
        self.fertile[index] = false;
        self.pregnant[index] = 0;

        let mate_index = self.mates[index];
        if mate_index != 0 {
            self.mates[mate_index] = 0;
            self.mates[index] = 0;
        }
    }

    // Helper method to convert 2D coordinates to a 1D index
    fn xy_to_index(&self, x: usize, y: usize) -> usize {
        1 + y * self.world_width + x
    }
    fn index_to_xy(&self, idx: usize) -> (usize, usize) {
        let idx_adjusted = idx - 1; // Adjust the index to account for the 1-based indexing
        let x = idx_adjusted % self.world_width;
        let y = idx_adjusted / self.world_width;
        (x, y)
    }

    pub fn move_to_position(&mut self, entity: usize, new_x: usize, new_y: usize) {
        // Retrieve the current position of the entity
        let current_position = self.positions[entity];
        
        // Remove the entity from its current position's residents set
        if let Some(residents) = self.residents.get_mut(&current_position) {
            residents.remove(&entity);
        }
    
        // Calculate the new position and update the entity's position
        let new_position = self.xy_to_index(new_x, new_y);
        self.positions[entity] = new_position;
    
        // Insert the entity into the new position's residents set
        self.residents.entry(new_position).or_insert_with(HashSet::new).insert(entity);
    }

    pub fn set_position(&mut self, entity: usize, x: usize, y: usize) {
        let position_id = self.xy_to_index(x, y);
        self.positions[entity] = position_id;
        self.residents.entry(position_id).or_insert_with(HashSet::new).insert(entity);
    }

    pub fn set_life(&mut self, entity: usize) {
        self.lives[entity] = true;
    }

    pub fn set_human(&mut self, entity: usize) {
        self.humans[entity] = true;
    }

    pub fn set_age(&mut self, entity: usize, date: usize) {
        self.spawn_dates[entity] = date;
    }

    pub fn set_sex(&mut self, entity: usize, sex: String) {
        if sex == "male" {
            self.males[entity] = true;
        } else {
            self.females[entity] = true;
        }
        
    }

    pub fn spawn_person(&mut self, x:usize, y: usize, age_days: usize, sex: String) -> usize {
        let entity = self.create_entity();
        self.set_position(entity, x, y);
        self.set_life(entity);
        self.set_human(entity);
        self.set_age(entity, age_days);
        self.set_sex(entity, sex);
        self.children.insert(entity, HashSet::new());
        entity
    }

    pub fn matchmaker_system(&mut self) {
        for (_position, residents) in &self.residents {
            let mut bachelors: Vec<usize> = Vec::new();
            let mut spinsters: Vec<usize> = Vec::new();

            // Populate bachelors and spinsters
            for &entity in residents.iter() {
                if self.humans[entity] && self.mates[entity] == 0 && (get_age(self.day, self.spawn_dates[entity]) >= 365*16){
                    if self.males[entity] {
                        bachelors.push(entity);
                    } else {
                        spinsters.push(entity);
                    }
                }
            }
            // Shuffle both groups to randomize pairing
            let mut rng = rand::thread_rng();

            // Determine the number of pairs to form based on the smaller group
            let mates = bachelors.into_iter()
                .zip(spinsters.into_iter())
                .filter(|_| rng.gen_bool(0.5))
                .flat_map(|(bachelor, spinster)| vec![(bachelor, spinster), (spinster, bachelor)]);

            for (a, b) in mates {
                self.mates[a] = b;
                self.mates[b] = a; 
            }

        }
    }

    pub fn fertility_system(&mut self) {
        let mut rng = rand::thread_rng();

        // Iterate through potential female entities to determine new fertile ones
        for (index, &is_female) in self.females.iter().enumerate() {
            // Check if the entity is human, female, not already fertile or pregnant, and within the age range
            if self.humans[index] && is_female && !self.fertile[index] && self.pregnant[index] == 0 {
                let age = get_age(self.day, self.spawn_dates[index]);
                if age >= 365 * 14 && age <= 365 * 50 {
                    if rng.gen_bool(0.015) {
                        self.fertile[index] = true;
                    }
                }
            }
        }
    }

    pub fn conception_system(&mut self) {
        let mut rng = rand::thread_rng();
        // Iterate through fertile entities to determine new conceptions
        for index in 0..self.humans.len() {
            if self.humans[index] && self.females[index] && self.fertile[index] && self.mates[index] != 0 && self.pregnant[index] == 0{
                // Assuming mates[index] != 0 means there is a mate (adjust logic as necessary)
                if rng.gen_bool(0.015) {
                    self.pregnant[index] = self.day + rng.gen_range(260..=300);
                    self.fertile[index] = false;
                }
            }
        }
    }

    pub fn birth_system(&mut self) {
        let mut rng = rand::thread_rng();

        for index in 0..self.humans.len() {
            // Check if the entity is pregnant and the due date has arrived
            if self.humans[index] && self.pregnant[index] != 0 && self.day >= self.pregnant[index] {
                // Reset pregnant status
                self.pregnant[index] = 0;
                // Determine sex of the newborn
                let sex = if rng.gen_bool(0.5) { "male" } else { "female" };
                let (x, y) = self.index_to_xy(index);
                let child_id = self.spawn_person(x, y, self.day, sex.to_owned());
                self.children.insert(child_id, HashSet::new());
                self.children.entry(index).or_insert_with(HashSet::new).insert(child_id);
                self.parents.insert(child_id, vec![index, self.mates[index]]);
                if self.mates[index] != 0 {
                    self.children.entry(self.mates[index]).or_insert_with(HashSet::new).insert(self.mates[index]);
                }
            }
        }
    }

    pub fn death_system(&mut self) {
        let mut rng = rand::thread_rng();

        // Iterate over all entities to check for death
        for index in 0..self.lives.len() {
            if self.humans[index] {
                let age = self.spawn_dates[index];
                let death_probability = calculate_death_probability(self.day, age);

                // Determine if the entity dies based on death probability
                if rng.gen_bool(death_probability) {
                    self.die(index); // Pass the index of the entity to die method
                }
            }
        }
            
    }

    pub fn time_system(&mut self) {
        self.day += 1;
    }

    pub fn move_system(&mut self) {
        let mut rng = rand::thread_rng();
        let mut moves = Vec::new();

        // Iterate over all positions
        for (position, residents) in &self.residents {
            // Determine the number of humans at this position
            let human_count = residents.iter().count();
            // Assuming there is at most one forage entity per position,
            // find the forage bounty associated with this position, if any.
            let forage_bounty = residents.iter()
                .filter_map(|e| Some(self.forages[*e]))
                .next()
                .map_or(1, |forage| forage);
            // Calculate the likelihood of moving based on human count and forage bounty
            let difference = human_count as i32 - forage_bounty as i32;
            let move_probability = 
            if human_count == 1 {
                0.9 // Very likely to move if alone
            }
            else if difference <= 0 {
                0.01 // Very unlikely to move if forage bounty is enough
            } else {
                0.01 + (difference as f64 * 0.1).min(0.19).clamp(0.0, 0.19) // Increasingly likely to move as difference grows
            };

            // Families Move
            for entity in residents.iter().filter(|e| self.humans[**e] && self.males[**e] && self.mates[**e] != 0) {
                if rng.gen_bool(move_probability) {
                    let (x, y) = self.index_to_xy(*position);
                    let movement = generate_random_move(&mut rng, x, y);
                    moves.push((*entity, (x as i32 + movement.0), y as i32 + movement.1));
                    if self.mates[*entity] != 0 {
                        moves.push((self.mates[*entity], (x as i32 + movement.0), y as i32 + movement.1));
                    }
                    for child in self.children[entity].iter() {
                        moves.push((*child, (x as i32 + movement.0), y as i32 + movement.1));
                    }
                }
            }
            // Singles Move
            for entity in residents.iter().filter(|e| self.humans[**e] && !self.mates[**e] != 0 && get_age(self.day, self.spawn_dates[**e]).ge(&(365 * 14))) {
                if rng.gen_bool(move_probability) {
                    let (x, y) = self.index_to_xy(*position);
                    let movement = generate_random_move(&mut rng, x, y);
                    moves.push((*entity, (x as i32 + movement.0), y as i32 + movement.1));
                    for child in self.children[entity].iter() {
                        moves.push((*child, (x as i32 + movement.0), y as i32 + movement.1));
                    }
                }
            }
        }

        // Apply the collected moves after determining all of them
        for (entity, new_x, new_y) in moves {
            self.move_to_position(entity, new_x as usize, new_y as usize);
        }
    }

    pub fn average_human_age(&self) -> usize {
        let today = self.day;
    
        // Sum the ages of all humans by subtracting their spawn date from the current day
        let total_age: usize = self.humans.iter().enumerate()
            .filter(|(_, &is_human)| is_human) // Keep only humans
            .map(|(index, _)| get_age(today, self.spawn_dates[index])) // Calculate each human's age
            .sum();
    
        // Count the number of humans
        let human_count: usize = self.humans.iter().filter(|&&is_human| is_human).count();
    
        // Calculate average age. Avoid division by zero by checking human_count.
        if human_count > 0 {
            total_age / human_count
        } else {
            0
        }
    }
}

fn calculate_death_probability(current_day: usize, birthdate: usize) -> f64 {
    let age_years = (current_day - birthdate) as f64 / 365.0; // Assuming 365 days per year for simplicity
    if age_years <= 18.0 {
        // Decay to 0.1 at age 18
        // Formula: a * exp(-b * x) + c, ensuring it's 1 at x=0 and 0.1 at x=18
        let a = 0.9;
        let b = -1.0 * (0.1_f64.ln() - 1.0_f64.ln()) / 18.0;
        let c = 0.1;
        return (a * (-b * age_years).exp() + c).clamp(0.0, 1.0);
    } else {
        // Grow to 2.0 by age 80
        // Formula: a * exp(b * x) + c, ensuring it's 0.1 at x=18 and 2 at x=80
        let x_shifted = age_years - 18.0; // Shift the x-axis to start at 0 for this phase
        let a = 1.9;
        let b = (2.0_f64.ln() - 0.1_f64.ln()) / (80.0 - 18.0);
        let c = 0.1 - a * (b * 18.0).exp(); // Adjust to ensure continuity at x=18
        return (a * (b * x_shifted).exp() + c).clamp(0.0, 1.0);
    }
}

fn get_age(day: usize, birthday: usize) -> usize {
    let age = day - birthday;
    age
}

fn generate_random_move(rng: &mut rand::rngs::ThreadRng, x:usize, y:usize) -> (i32, i32) {
    let center_reference = 50;
    let mut move_x;
    let mut move_y;

    // Adjust movement probability based on distance from center_reference
    if x < center_reference {
        // More likely to move right if left of center
        move_x = if rng.gen_bool(0.75) { 1 } else { -1 };
    } else if x > center_reference {
        // More likely to move left if right of center
        move_x = if rng.gen_bool(0.75) { -1 } else { 1 };
    } else {
        // Equally likely to move in any direction if exactly at center_reference
        move_x = if rng.gen_bool(0.5) { 1 } else { -1 };
    }

    // Similar logic applied to y-axis
    if y < center_reference {
        // More likely to move up if below center
        move_y = if rng.gen_bool(0.75) { 1 } else { -1 };
    } else if y > center_reference {
        // More likely to move down if above center
        move_y = if rng.gen_bool(0.75) { -1 } else { 1 };
    } else {
        // Equally likely to move in any direction if exactly at center_reference
        move_y = if rng.gen_bool(0.5) { 1 } else { -1 };
    }

    // If both move_x and move_y are 0 (which shouldn't happen with above logic), force a move
    if move_x == 0 && move_y == 0 {
        move_x = if rng.gen_bool(0.5) { 1 } else { -1 };
        move_y = if rng.gen_bool(0.5) { 1 } else { -1 };
    }
    
    (move_x, move_y)
}

