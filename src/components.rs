#[derive(PartialEq, Eq, Hash, Clone, Copy, Default)]
pub struct Position(pub u32);

#[derive(Clone, Copy, Default)]
pub struct Life(pub bool);

#[derive(Clone, Copy, Default)]
pub struct Human(pub bool);
#[derive(Clone, Copy, Default)]
pub struct SpawnDate (pub u32);
#[derive(Clone, Copy, Default)]
pub struct Forage(pub u8);
#[derive(Clone, Copy, Default)]
pub struct Male(pub bool);

#[derive(Clone, Copy, Default)]
pub struct Female(pub bool);

#[derive(Clone, Copy, Default)]
pub struct Fertile(pub bool);

#[derive(Clone, Copy, Default)]
pub struct Pregnant(pub u32);
