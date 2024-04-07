use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use crate::components::{Position, Life, SpawnDate, Sex, Human, Forage};
use crate::entity::Entity;

pub struct World {
    pub day: u32,
    pub entities: u32,
    pub positions: HashMap<Entity, Position>,
    pub lives: HashMap<Entity, Life>,
    pub ages: HashMap<Entity, SpawnDate>,
    pub sexes: HashMap<Entity, Sex>,
    pub humans: HashMap<Entity, Human>,
    pub forages: HashMap<Entity, Forage>,
    pub residents: HashMap<Position, HashSet<Entity>>,
    pub mates: HashMap<Entity, Entity>,

}

impl World {
    pub fn new() -> Self {
        World {
            entities: 0,
            day: 0,
            positions: HashMap::new(),
            lives: HashMap::new(),
            ages: HashMap::new(),
            sexes: HashMap::new(),
            humans: HashMap::new(),
            forages: HashMap::new(),
            residents: HashMap::new(),
            mates: HashMap::new(),
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

    pub fn add_sex(&mut self, entity: Entity, sex: Sex) {
        self.sexes.insert(entity, sex);
    }

    pub fn spawn_person(&mut self, x:i32, y: i32, age_days: i32, sex: Sex) -> Entity {
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
                if self.humans.contains_key(&entity) && !self.mates.contains_key(&entity) && (get_age(self.day, self.ages.get(&entity).unwrap().date) >= 365*16) {
                    match self.sexes.get(&entity) {
                        Some(Sex::Male) => bachelors.push(entity),
                        Some(Sex::Female) => spinsters.push(entity),
                        None => (),
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
        for &entity in self.humans.iter().filter(|e| matches!(self.sexes.get(e), Some(Sex::Female)) && (get_age(self.day, self.ages.get(&entity).unwrap().date) >= 365*14))  {
            
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

            //Singles move
            for entity in residents.iter().filter(|e| self.humans.contains_key(e) && matches!(self.sexes.get(e), Some(Sex::Male))) {
                if rng.gen_bool(move_probability) {
                    let movement = generate_random_move(&mut rng);
                    moves.push((*entity, position.x + movement.0, position.y + movement.1));
                    self.mates.get(entity).inspect(|mate| moves.push((**mate, position.x + movement.0, position.y + movement.1)));
                }
            }
            for entity in residents.iter().filter(|e| self.humans.contains_key(e) && matches!(self.sexes.get(e), Some(Sex::Female)) && !self.mates.contains_key(e)) {
                if rng.gen_bool(move_probability) {
                    let movement = generate_random_move(&mut rng);
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

fn generate_random_move(mut rng: &mut ThreadRng) -> (i32, i32) {
    if rng.gen_bool(0.5) {
        if rng.gen_bool(0.5) {
            (1, 0) // Move right
        } else {
            (-1, 0) // Move left
        }
    } else {
        if rng.gen_bool(0.5) {
            (0, 1) // Move up
        } else {
            (0, -1) // Move down
        }
    }
}
