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
    let height = 100;
    let width = 100;

    let mut world = world::World::new(100, 100);
    let mut rng = rand::thread_rng();
    // Spawn 100 living entities at (0, 0)
    let pop = 100;
    for _ in 0..pop {
        let sex = if rng.gen_bool(0.5) { "male" } else { "female" };
        let age = 365*100 - rng.gen_range(18*365..=30*365);
        world.spawn_person(width / 2, height / 2, age, sex.to_owned());
    }
    loop {
        world.time_system();
        world.matchmaker_system();
        world.fertility_system();
        world.conception_system();
        world.birth_system();
        world.death_system();
        world.move_system();
        clear_console();
        
        println!("Year {}, Day {}, Humans {}, Mean Age {}, Males {}, Women {}, Coupled {}, Pregnant {}", 
        1 + world.day / 365, 
        world.day % 365, 
        world.humans.iter().filter(|&&is_human| is_human).count(), 
        world.average_human_age()/365,
        world.males.iter().filter(|&&is_male| is_male).count(), 
        world.females.iter().filter(|&&is_female| is_female).count(), 
        world.mates.iter().filter(|&&mate| mate != 0).count(), 
        world.pregnant.iter().filter(|&&is_pregnant| is_pregnant != 0).count());
        
        // let mut counts = vec![0; width*height];

        // world.humans.iter().enumerate()
        //     .filter(|&(_index, &is_human)| is_human) // Filter to keep only humans
        //     .for_each(|(index, _)| {
        //         // Subtract 1 from position to convert from 1-based to 0-based indexing
        //         let position_index = world.positions[index] - 1;
        //         if position_index < counts.len() {
        //             // Increment the count for the human's position
        //             counts[position_index] += 1;
        //         }
        //     });

        // // Find the maximum count for scaling the gradient
        // let max_count = 20;//counts.iter().flatten().max().copied().unwrap_or(0);

        // for y in 0..height {
        //     for x in 0..width {
        //         // Calculate the index for one-indexed flat array, adjusting for 0-based indexing in Rust
        //         let index = y * width + x; // This calculation suits a zero-indexed array
        //         let count = counts[index]; // Access the count at this position
        
        //         // Determine the color based on 'count'
        //         let color = if count == 0 {
        //             RGB(255, 255, 255) // White for empty
        //         } else {
        //             let fraction = count as f32 / max_count as f32;
        //             if fraction < 0.5 {
        //                 // Transition from white to green
        //                 let green = 255; // Max green the whole time
        //                 let red_blue = 255 - (fraction * 2.0 * 255.0) as u8; // Decrease red and blue
        //                 RGB(red_blue, green, red_blue)
        //             } else {
        //                 // Transition from green to red
        //                 let green = 255 - ((fraction - 0.5) * 2.0 * 255.0) as u8; // Decrease green
        //                 let red = 255; // Max red the whole time
        //                 RGB(red, green, 0)
        //             }
        //         };
        
        //         // Print a colored block without any space between them
        //         print!("{}", color.paint("██"));
        //     }
        //     println!(); // New line for the next row
        // }
    }
}