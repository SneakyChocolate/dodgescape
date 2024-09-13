use crate::{action::Action, game::Game, vector};


#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub active: bool,
    pub effects: Vec<ItemEffect>,
}

#[derive(Debug)]
pub enum ItemEffect {
    Vision((f32,f32)),
    Speed(f32),
    SlowEnemies(f32, f32),
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
                    ItemEffect::SlowEnemies(s, radius) => {
                        for group in game.enemies.iter_mut() {
                            for enemy in group.1.iter_mut() {
                                if vector::distance((player.x, player.y), (enemy.x, enemy.y)).2 - enemy.radius <= *radius {
                                    let original = vector::abs(enemy.velocity);
                                    enemy.effects.push(crate::enemy::EnemyEffect::SpeedAlter { original, new: original * *s, ease: 0 });
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
    pub fn new(name: &str, amount: usize, effects: Vec<ItemEffect>) -> Self {
        Item {
            name: name.to_owned(),
            active: false,
            effects,
        }
    }
}
