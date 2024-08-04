use crate::{enemy::{Effect, Enemy}, game::{Game}, vector};

pub enum Action {
    UpdateEnemyVelocity(usize, (f32,f32)),
    UpdatePlayerVelocity((f32,f32)),
    AddPlayerVelocity((f32,f32)),
    SpawnCrumble(usize),
    ReduceLifetime(usize),
    ReduceCooldown(usize),
    Despawn(usize),
    SpawnProjectile { group: usize, velocity: (f32, f32), radius: f32, color: String },
}

impl Action {
    pub fn execute(&self, game: &mut Game, entity: usize) {
        match self {
            Action::UpdateEnemyVelocity(g, v) => {
                let enemy = game.enemies.get_mut(*g).unwrap().1.get_mut(entity).unwrap();
                enemy.velocity = *v;
            },
            Action::UpdatePlayerVelocity(v) => {
                let object = game.players.get_mut(entity).unwrap();
                object.velocity = *v;
            },
            Action::AddPlayerVelocity(v) => {
                let object = game.players.get_mut(entity).unwrap();
                object.velocity = (object.velocity.0 + v.0, object.velocity.1 + v.1);
            },
            Action::SpawnCrumble(g) => {
                let enemy = game.enemies.get_mut(*g).unwrap().1.get_mut(entity).unwrap();
                let (x, y, v, r) = (enemy.x, enemy.y, enemy.velocity.clone(), enemy.radius / 2.0);
                // cumble
                let mut crumble = Enemy::new(x, y, vector::normalize(v, 0.5), r, "rgb(0,0,0)");
                crumble.effects.push(Effect::Lifetime(2000));
                game.enemies.get_mut(*g).unwrap().1.push(crumble);
            },
            Action::Despawn(g) => {
                game.enemies.get_mut(*g).unwrap().1.remove(entity);
            },
            Action::ReduceLifetime(g) => {
                // TODO why does unwrap throw an error?
                let enemy = match game.enemies.get_mut(*g).unwrap().1.get_mut(entity) {
                    Some(e) => e,
                    None => {return;},
                };
                let mut despawn: Vec<(usize, usize)> = vec![];
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::Effect::Lifetime(t) => {
                            if *t == 0 {
                                despawn.push((*g, entity));
                            }
                            else {
                                *effect = crate::enemy::Effect::Lifetime(*t - 1);
                            }
                        },
                        _ => {},
                    }
                }
                for (g, i) in despawn {
                    game.enemies.get_mut(g).unwrap().1.remove(i);
                }
            },
            Action::ReduceCooldown(group) => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::Effect::Shoot { radius, speed, time_left, cooldown } => {
                            if *time_left == 0 {
                                *time_left = *cooldown;
                            }
                            else {
                                *time_left -= 1;
                            }
                        },
                        _ => {},
                    }
                }
            },
            Action::SpawnProjectile { group, velocity, color, radius } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                // projectile
                let mut crumble = Enemy::new(enemy.x, enemy.y, *velocity, *radius, color.as_str());
                crumble.effects.push(Effect::Lifetime(1000));
                game.enemies.get_mut(*group).unwrap().1.push(crumble);
            },
        }
    }
}

