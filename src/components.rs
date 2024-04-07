#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Life {
}


pub struct Human {
}

pub struct SpawnDate {
    pub date: i32, // Age in days
}

pub struct Forage {
    pub bounty: u32,
}

pub struct Male;
pub struct Female;

pub struct Fertile;

pub struct Pregnant {
    pub due_date: u32,
}
