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
    }
}