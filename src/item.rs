use crate::{action::Action, game::{DrawPack, Game}, gametraits::{Drawable, Radius}, vector};


#[derive(Debug, Default)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub active: bool,
    pub effects: Vec<ItemEffect>,
    pub drawpacks: Vec<DrawPack>,
    pub icon: Option<DrawPack>,
}

#[derive(Debug)]
pub enum ItemEffect {
    Vision((f32,f32)),
    Speed(f32),
    SlowEnemies{power: f32, radius: Radius, duration: usize},
    ShrinkEnemies{power: f32, radius: Radius, duration: usize},
}

pub fn handle_effects(game: &mut Game) {
    let mut actions: Vec<(usize, Action)> = vec![];
    for (p, player) in game.players.iter().enumerate() {
        for item in player.inventory.items.iter() {
            if !item.active {continue;}
            for effect in item.effects.iter() {
                match effect {
                    ItemEffect::Vision(zoom) => {
                        actions.push((p, Action::SetPlayerZoomlimit(*zoom)));
                    },
                    ItemEffect::Speed(s) => {
                        actions.push((p, Action::MulPlayerSpeed(*s)));
                    },
                    ItemEffect::SlowEnemies{power, radius, duration } => {
                        for group in game.enemies.iter_mut() {
                            for enemy in group.1.iter_mut() {
                                if vector::distance((player.x, player.y), (enemy.x, enemy.y)).2 - enemy.get_radius() <= radius.translate(player.get_radius()) {
                                    // check if effect of this item id is already applied
                                    let effect = enemy.effects.iter_mut().find(|e| {
                                        match e {
                                            crate::enemy::EnemyEffect::SpeedAlter { origin, power: slow, ease } => {
                                                *origin == item.id
                                            },
                                            _ => {
                                                false
                                            }
                                        }
                                    });
                                    match effect {
                                        Some(e) => {
                                            match e {
                                                crate::enemy::EnemyEffect::SpeedAlter { origin, power: slow, ease } => {
                                                    *ease = *duration;
                                                },
                                                _ => {
                                                    // do nothing
                                                }
                                            }
                                        },
                                        None => {
                                            enemy.effects.push(crate::enemy::EnemyEffect::SpeedAlter { power: *power, ease: *duration, origin: item.id });
                                        },
                                    }
                                }
                            }
                        }
                    },
                    ItemEffect::ShrinkEnemies{power, radius, duration } => {
                        for group in game.enemies.iter_mut() {
                            for enemy in group.1.iter_mut() {
                                if vector::distance((player.x, player.y), (enemy.x, enemy.y)).2 - enemy.get_radius() <= radius.translate(player.get_radius()) {
                                    // check if effect of this item id is already applied
                                    let effect = enemy.effects.iter_mut().find(|e| {
                                        match e {
                                            crate::enemy::EnemyEffect::Shrink { origin, power, ease } => {
                                                *origin == item.id
                                            },
                                            _ => {
                                                false
                                            }
                                        }
                                    });
                                    match effect {
                                        Some(e) => {
                                            match e {
                                                crate::enemy::EnemyEffect::Shrink { origin, power, ease } => {
                                                    *ease = *duration;
                                                },
                                                _ => {
                                                    // do nothing
                                                }
                                            }
                                        },
                                        None => {
                                            enemy.effects.push(crate::enemy::EnemyEffect::Shrink { power: *power, ease: *duration, origin: item.id });
                                        },
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
        actions.push((p, Action::SetPlayerZoomlimit((1.0, 1.0))));
        actions.push((p, Action::SetPlayerSpeed(8.0)));
    }
    for (entity, action) in actions.iter().rev() {
        action.execute(game, *entity);
    }
}

impl Item {
    pub fn new(name: &str, effects: Vec<ItemEffect>, drawpacks: Vec<DrawPack>, item_counter: &mut usize, icon: Option<DrawPack>) -> Self {
        let item = Item {
            name: name.to_owned(),
            active: false,
            effects,
            drawpacks,
            id: *item_counter,
            icon,
        };
        *item_counter += 1;
        item
    }
}
