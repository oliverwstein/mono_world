use ansi_term::Colour::RGB;
use rand::Rng;
use std::process::Command;

mod world; mod entity; mod components;

fn clear_console() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "cls"])
            .status()
            .unwrap();
    } else {
        // Assume Unix-like
        Command::new("clear")
            .status()
            .unwrap();
    }
}

fn main() {
    let mut world = world::World::new();
    let mut rng = rand::thread_rng();
    // Spawn 100 living entities at (0, 0)
    for _ in 0..10000 {
        let age_days = -1*rng.gen_range(18*365..=30*365);
        let sex = if rng.gen_bool(0.5) { "male" } else { "female" };
        world.spawn_person(0, 0, age_days, sex.to_owned());
    }
    world.spawn_resources(0, 0, rng.gen_range(1..=10));
    

    loop {
        world.time_system();
        world.matchmaker_system();
        world.fertility_system();
        world.conception_system();
        world.birth_system();
        world.death_system();
        world.move_system();
        clear_console();
        
        println!("Year {}, Day {}, Humans {}, Males {}, Women {}, Coupled {}, Pregnant {}", 1 + world.day / 365, world.day % 365, world.humans.len(), world.males.len(), world.females.len(), world.mates.len(), world.pregnant.len());
        let mut min_x = 0;
        let mut max_x = 0;
        let mut min_y = 0;
        let mut max_y = 0;

        for position in world.positions.values() {
            min_x = min_x.min(position.x);
            max_x = max_x.max(position.x);
            min_y = min_y.min(position.y);
            max_y = max_y.max(position.y);
        }

        // Create a 2D array to hold the counts, initialized to 0
        let width = (max_x - min_x + 1) as usize;
        let height = (max_y - min_y + 1) as usize;
        let mut counts = vec![vec![0; width]; height];

        // Populate the array with entity counts
        for (entity, position) in &world.positions {
            if let Some(_life) = world.lives.get(entity) {
                let x_index = (position.x - min_x) as usize;
                let y_index = (position.y - min_y) as usize;
                counts[y_index][x_index] += 1;
            }
        }
        // Find the maximum count for scaling the gradient
        let max_count = 20;//counts.iter().flatten().max().copied().unwrap_or(0);

        for row in &counts {
            for &count in row {
                let color = if count == 0 {
                    RGB(255, 255, 255) // White for empty
                } else {
                    let fraction = count as f32 / max_count as f32;
                    if fraction < 0.5 {
                        // Transition from white to green
                        let green = 255; // Max green the whole time
                        let red_blue = 255 - (fraction * 2.0 * 255.0) as u8; // Decrease red and blue
                        RGB(red_blue, green, red_blue)
                    } else {
                        // Transition from green to red
                        let green = 255 - ((fraction - 0.5) * 2.0 * 255.0) as u8; // Decrease green
                        let red = 255; // Max red the whole time
                        RGB(red, green, 0)
                    }
                };
                // Print a colored block without any space between them
                print!("{}", color.paint("██"));
            }
            println!(); // New line for the next row
        }

    }
}