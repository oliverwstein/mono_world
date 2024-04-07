use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use crate::components::{Fertile, Forage, Human, Life, Position, Pregnant, Male, Female, SpawnDate};
use crate::entity::Entity;

pub struct World {
    pub day: u32,
    pub entities: u32,
    pub positions: HashMap<Entity, Position>,
    pub lives: HashMap<Entity, Life>,
    pub ages: HashMap<Entity, SpawnDate>,
    pub males: HashMap<Entity, Male>,
    pub females: HashMap<Entity, Female>,
    pub humans: HashMap<Entity, Human>,
    pub forages: HashMap<Entity, Forage>,
    pub residents: HashMap<Position, HashSet<Entity>>,
    pub mates: HashMap<Entity, Entity>,
    pub parents: HashMap<Entity, Vec<Entity>>,
    pub children: HashMap<Entity, Vec<Entity>>,
    pub fertile: HashMap<Entity, Fertile>,
    pub pregnant: HashMap<Entity, Pregnant>,
}


impl World {
    pub fn new() -> Self {
        World {
            entities: 0,
            day: 0,
            positions: HashMap::new(),
            lives: HashMap::new(),
            ages: HashMap::new(),
            males: HashMap::new(),
            females: HashMap::new(),
            humans: HashMap::new(),
            forages: HashMap::new(),
            residents: HashMap::new(),
            mates: HashMap::new(),
            fertile: HashMap::new(),
            pregnant: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
    
    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.entities);
        self.entities += 1;

        entity
    }

    pub fn add_position(&mut self, entity: Entity, x: i32, y: i32) {
        self.positions.insert(entity, Position { x, y });
        self.residents.entry(Position { x, y })
        .or_insert_with(HashSet::new)
        .insert(entity);
    }
    
    pub fn move_to_position(&mut self, entity: Entity, new_x: i32, new_y: i32) {
        // Initial setup remains unchanged
        let mut spawn_resource_here = None;
    
        if let Some(old_position) = self.positions.get(&entity) {
            if let Some(entities_at_old_pos) = self.residents.get_mut(old_position) {
                entities_at_old_pos.remove(&entity);
    
                // Check if the old position is now empty
                if entities_at_old_pos.is_empty() {
                    // Mark this position for resource spawning
                    spawn_resource_here = Some(*old_position);
                }
            }
        }
        self.positions.remove(&entity);
        self.positions.insert(entity, Position { x: new_x, y: new_y });
        self.residents.entry(Position { x: new_x, y: new_y })
            .or_insert_with(HashSet::new)
            .insert(entity);
    
        // After ensuring no borrows are held, spawn resources if needed
        if let Some(position_to_spawn) = spawn_resource_here {
            let mut rng = rand::thread_rng();
            self.spawn_resources(position_to_spawn.x, position_to_spawn.y, rng.gen_range(1..=10));
        }
    }
    

    pub fn add_life(&mut self, entity: Entity) {
        self.lives.insert(entity, Life {});
    }

    pub fn add_human(&mut self, entity: Entity) {
        self.humans.insert(entity, Human {});
    }

    pub fn add_age(&mut self, entity: Entity, date: i32) {
        self.ages.insert(entity, SpawnDate { date });
    }

    pub fn add_sex(&mut self, entity: Entity, sex: String) {
        if sex == "male" {
            self.males.insert(entity, Male);
        } else {
            self.females.insert(entity, Female);
        }
        
    }

    pub fn spawn_person(&mut self, x:i32, y: i32, age_days: i32, sex: String) -> Entity {
        let entity = self.create_entity();
        self.add_position(entity, x, y);
        self.add_life(entity);
        self.add_human(entity);
        self.add_age(entity, age_days);
        self.add_sex(entity, sex);
        entity
    }

    pub fn add_forage(&mut self, entity: Entity, bounty: u32) {
        self.forages.insert(entity, Forage { bounty });
    }

    pub fn spawn_resources(&mut self, x:i32, y: i32, bounty: u32) -> Entity {
        let entity = self.create_entity();
        self.add_position(entity, x, y);
        self.add_forage(entity, bounty);
        entity
    }

    pub fn matchmaker_system(&mut self) {
        for (_position, residents) in &self.residents {
            let mut bachelors: Vec<Entity> = Vec::new();
            let mut spinsters: Vec<Entity> = Vec::new();

            // Populate bachelors and spinsters
            for &entity in residents.iter() {
                if self.humans.contains_key(&entity) && !self.mates.contains_key(&entity) && (get_age(self.day, self.ages.get(&entity).unwrap().date) >= 365*16){
                    if self.males.contains_key(&entity) {
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
                .filter(|_| rng.gen_bool(0.25))
                .flat_map(|(bachelor, spinster)| vec![(bachelor, spinster), (spinster, bachelor)]);

            self.mates.extend(mates);

        }
    }

    pub fn fertility_system(&mut self) {
        let mut rng = rand::thread_rng();

        let new_fertile: Vec<_> = self.humans.iter()
            .map(|(e,_ )| *e)
            .filter(|entity| self.females.contains_key(entity) && !self.fertile.contains_key(entity) && !self.pregnant.contains_key(entity) && get_age(self.day, self.ages.get(entity).unwrap().date).ge(&(365 * 14)) && get_age(self.day, self.ages.get(entity).unwrap().date).le(&(365 * 50)))
            .filter(|entity| rng.gen_bool(0.015 / (self.children.get(entity).map_or(1.0, |children| children.len().max(1) as f64)/2.0)))
            .map(|entity| (entity, Fertile))
            .collect();

        self.fertile.extend(new_fertile);
    }

    pub fn conception_system(&mut self) {
        let conceptions: Vec<_> = self.humans.iter()
            .map(|(e,_ )| *e)
            .filter(|entity| self.fertile.contains_key(entity) && self.mates.contains_key(entity))
            .filter(|_| rand::thread_rng().gen_bool(0.03) )
            .map(|entity| (entity, Pregnant { due_date: self.day + rand::thread_rng().gen_range(260..=300) }))
            .collect();

        for (e, _p) in conceptions.iter(){self.fertile.remove(e);}
        self.pregnant.extend(conceptions);
    }

    pub fn birth_system(&mut self) {
        let births: Vec<_> = self.humans.iter()
            .map(|(e,_ )| *e)
            .filter(|entity| self.pregnant.contains_key(entity) && self.day.ge(&self.pregnant[entity].due_date))
            .collect();

        for e in births.iter(){
            self.pregnant.remove(e);
            let sex = if rand::thread_rng().gen_bool(0.5) { "male" } else { "female" };
            let child = self.spawn_person(self.positions[e].x, self.positions[e].y, self.day as i32, sex.to_owned());
            self.parents.insert(child, vec![*e, self.mates[e]]);
            self.children.entry(*e)
                .or_insert_with(Vec::new)
                .push(child);
            self.children.entry(self.mates[e])
                .or_insert_with(Vec::new)
                .push(child);

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
            let human_count = residents.iter().filter(|e| self.humans.contains_key(e)).count();
            // Assuming there is at most one forage entity per position,
            // find the forage bounty associated with this position, if any.
            let forage_bounty = residents.iter()
                .filter_map(|e| self.forages.get(e))
                .next()
                .map_or(1, |forage| forage.bounty);
            // Calculate the likelihood of moving based on human count and forage bounty
            let difference = human_count as i32 - forage_bounty as i32;
            let move_probability = 
            if difference <= 0 {
                0.01 // Very unlikely to move if forage bounty is enough
            } else {
                0.01 + (difference as f64 * 0.1).min(0.19) // Increasingly likely to move as difference grows
            };

            // Families move
            for entity in residents.iter().filter(|e| self.humans.contains_key(e) && self.males.contains_key(e)) {
                if rng.gen_bool(move_probability) {
                    let movement = generate_random_move(&mut rng, *position);
                    moves.push((*entity, position.x + movement.0, position.y + movement.1));
                    self.mates.get(entity).inspect(|mate| moves.push((**mate, position.x + movement.0, position.y + movement.1)));
                    if let Some(children) = self.children.get(entity) {
                        for &child_id in children {
                            moves.push((child_id, position.x + movement.0, position.y + movement.1));
                        }
                    }
                }
            }
            for entity in residents.iter().filter(|e| self.humans.contains_key(e) && self.females.contains_key(e) && !self.mates.contains_key(e)) {
                if rng.gen_bool(move_probability) {
                    let movement = generate_random_move(&mut rng, *position);
                    moves.push((*entity, position.x + movement.0, position.y + movement.1));
                }
            }
        }

        // Apply the collected moves after determining all of them
        for (entity, new_x, new_y) in moves {
            self.move_to_position(entity, new_x, new_y);
        }
    }

    
}

fn get_age(day: u32, birthday: i32) -> u32 {
    let age = day as i32 - birthday;
    age as u32
}

fn generate_random_move(mut rng: &mut ThreadRng, position: Position) -> (i32, i32) {
    let mut move_x = 0;
    let mut move_y = 0;
    
    // Adjust movement probability based on x position
    if position.x <= -50 {
        move_x = 1; // Must move right
    } else if position.x >= 50 {
        move_x = -1; // Must move left
    } else {
        // Probability decreases as it gets further from center
        let prob_move_right = 0.5 - (position.x as f64 / 100.0); // Increase as x decreases
        
        if rng.gen_bool(prob_move_right) {
            move_x = 1;
        } else if rng.gen_bool(1.0-prob_move_right) {
            move_x = -1;
        }
    }
    
    // Adjust movement probability based on y position
    if position.y <= -50 {
        move_y = 1; // Must move up
    } else if position.y >= 50 {
        move_y = -1; // Must move down
    } else {
        // Similar to x, but for y position
        let prob_move_up = 0.5 - (position.y as f64 / 100.0);        
        if rng.gen_bool(prob_move_up) {
            move_y = 1;
        } else if rng.gen_bool(1.0 - prob_move_up) {
            move_y = -1;
        }
    }

    // If both move_x and move_y are 0, force a move in any direction
    if move_x == 0 && move_y == 0 {
        if rng.gen_bool(0.5) { move_x = if rng.gen_bool(0.5) { 1 } else { -1 }; }
        else { move_y = if rng.gen_bool(0.5) { 1 } else { -1 }; }
    }
    
    (move_x, move_y)
}
