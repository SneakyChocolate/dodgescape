use crate::{action::Action, game::{DrawPack, Game}, vector};


#[derive(Debug, Default)]
pub struct Item {
    pub name: String,
    pub active: bool,
    pub effects: Vec<ItemEffect>,
    pub drawpacks: Vec<DrawPack>,
}

#[derive(Debug)]
pub enum ItemEffect {
    Vision((f32,f32)),
    Speed(f32),
    SlowEnemies{slow: f32, radius: f32, duration: usize},
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
                    ItemEffect::SlowEnemies{slow, radius, duration } => {
                        for group in game.enemies.iter_mut() {
                            for enemy in group.1.iter_mut() {
                                if vector::distance((player.x, player.y), (enemy.x, enemy.y)).2 - enemy.radius <= *radius {
                                    // if enemy doesnt have effect already, else set ease to 1
                                    if enemy.effects.iter_mut().all(|e| {
                                        match e {
                                            crate::enemy::EnemyEffect::SpeedAlter { slow: applied_slow, ease } => {
                                                *ease = *duration;
                                                if *slow < *applied_slow {
                                                    *applied_slow = *slow;
                                                }
                                                false
                                            },
                                            _ => {
                                                true
                                            }
                                        }
                                    }) {
                                        enemy.effects.push(crate::enemy::EnemyEffect::SpeedAlter { slow: *slow, ease: *duration });
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
    pub fn new(name: &str, effects: Vec<ItemEffect>, drawpacks: Vec<DrawPack>) -> Self {
        Item {
            name: name.to_owned(),
            active: false,
            effects,
            drawpacks,
        }
    }
}
