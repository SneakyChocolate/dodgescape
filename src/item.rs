use crate::{action::Action, game::{DrawPack, Game}, gametraits::{Drawable, Radius}, vector};
use crate::gametraits::*;


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
    Revive{radius: Radius},
    Consumable{uses: usize},
    PushEnemies{power: f32, radius: Radius},
    RotateEnemies{power: f32, radius: Radius},
    Harden{limit: usize, cooldown: usize},
    Usable,
}

pub fn handle_effects(game: &mut Game) {
    let mut actions: Vec<(usize, Action)> = vec![];
    let mut deletions: Vec<(usize, Action)> = vec![];
    for (p, player) in game.players.iter().enumerate() {
        for (i, item) in player.inventory.items.iter().enumerate() {
            if !item.active {continue;}
            for (e, effect) in item.effects.iter().enumerate() {
                match effect {
                    ItemEffect::Vision(zoom) => {
                        actions.push((p, Action::SetPlayerZoomlimit(*zoom)));
                    },
                    ItemEffect::Speed(s) => {
                        actions.push((p, Action::MulPlayerSpeedMultiplier { f: *s }));
                    },
                    ItemEffect::SlowEnemies{power, radius, duration } => {
                        for group in game.enemies.iter_mut() {
                            for enemy in group.1.iter_mut() {
                                if vector::distance((player.get_x(), player.get_y()), (enemy.get_x(), enemy.get_y())).2 - enemy.get_radius() <= radius.translate(player.get_radius()) {
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
                                if vector::distance((player.get_x(), player.get_y()), (enemy.get_x(), enemy.get_y())).2 - enemy.get_radius() <= radius.translate(player.get_radius()) {
                                    // check if effect of this item id is already applied
                                    let effect = enemy.effects.iter_mut().find(|e| {
                                        match e {
                                            crate::enemy::EnemyEffect::Shrink { origin, power, ease, start } => {
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
                                                crate::enemy::EnemyEffect::Shrink { origin, power, ease, start } => {
                                                    *ease = *duration;
                                                },
                                                _ => {
                                                    // do nothing
                                                }
                                            }
                                        },
                                        None => {
                                            enemy.effects.push(crate::enemy::EnemyEffect::Shrink { power: *power, ease: *duration, origin: item.id, start: *duration });
                                        },
                                    }
                                }
                            }
                        }
                    },
                    ItemEffect::Revive { radius } => {
                        actions.push((p, Action::RevivePlayers { radius: *radius }));
                    },
                    ItemEffect::Consumable { uses } => {
                        if *uses == 0 {
                            deletions.push((p, Action::RemovePlayerItem { item: i }));
                        }
                        else {
                            deletions.push((p, Action::DecreaseItemEffect { item: i, effect: e }));
                        }
                    },
                    ItemEffect::PushEnemies { power, radius } => {
                        for (g, group) in game.enemies.iter().enumerate() {
                            for (e, enemy) in group.1.iter().enumerate() {
                                let dist = vector::distance((player.get_x(), player.get_y()), (enemy.get_x(), enemy.get_y()));
                                if dist.2 <= radius.translate(player.get_radius()) + enemy.get_radius() {
                                    let add = vector::normalize((dist.0, dist.1), *power);
                                    actions.push((e, Action::AddEnemyPosition { group: g, x: add.0, y: add.1 }));
                                }
                            }
                        }
                    },
                    ItemEffect::RotateEnemies { power, radius } => {
                        for (g, group) in game.enemies.iter().enumerate() {
                            for (e, enemy) in group.1.iter().enumerate() {
                                let dist = vector::distance((player.get_x(), player.get_y()), (enemy.get_x(), enemy.get_y()));
                                if dist.2 <= radius.translate(player.get_radius()) + enemy.get_radius() {
                                    let angle = vector::angle_from_point((dist.0, dist.1));
                                    let newu = vector::point_from_angle(angle + *power);
                                    let newp = vector::normalize((newu.0, newu.1), dist.2);
                                    let new = (player.get_x() + newp.0, player.get_y() + newp.1);
                                    let add = (new.0 - enemy.get_x(), new.1 - enemy.get_y());
                                    actions.push((e, Action::AddEnemyPosition { group: g, x: add.0, y: add.1 }));
                                }
                            }
                        }
                    },
                    ItemEffect::Harden { limit, cooldown } => {
                        let effect = player.effects.iter().position(|e| {
                            match e {
                                crate::player::PlayerEffect::Harden { ease, cooldown } => true,
                                _ => false,
                            }
                        });
                        match effect {
                            Some(effect) => {
                                
                            },
                            None => {
                                actions.push((p, Action::PushPlayerEffect(crate::player::PlayerEffect::Harden { ease: *limit, cooldown: *cooldown })));
                            },
                        }
                    },
                    ItemEffect::Usable => {
                        actions.push((p, Action::SetItemActive { i, v: false }));
                    },
                }
            }
        }
    }
    for (entity, action) in actions.iter().rev() {
        action.execute(game, *entity);
    }
    for (entity, action) in deletions.iter().rev() {
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
