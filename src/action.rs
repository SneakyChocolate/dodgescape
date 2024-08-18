use core::panic;

use crate::{enemy::{EnemyEffect, Enemy}, game::{Game}, vector};

pub enum Action {
    UpdateEnemyVelocity(usize, (f32,f32)),
    SetPlayerVelocity((f32,f32)),
    AddPlayerVelocity((f32,f32)),
    MulPlayerVelocity(f32),
    SpawnCrumble(usize),
    ReduceLifetime(usize),
    ReduceCooldown(usize),
    ResetCooldown(usize),
    Despawn(usize),
    SpawnProjectile { group: usize, velocity: (f32, f32), radius: f32, color: String, lifetime: usize },
}

impl Action {
    pub fn execute(&self, game: &mut Game, entity: usize) {
        match self {
            Action::UpdateEnemyVelocity(g, v) => {
                let enemy = game.enemies.get_mut(*g).unwrap().1.get_mut(entity).unwrap();
                enemy.velocity = *v;
            },
            Action::SetPlayerVelocity(v) => {
                let object = game.players.get_mut(entity).unwrap();
                object.velocity = *v;
            },
            Action::AddPlayerVelocity(v) => {
                let object = game.players.get_mut(entity).unwrap();
                object.velocity = (object.velocity.0 + v.0, object.velocity.1 + v.1);
            },
            Action::MulPlayerVelocity(factor) => {
                let object = game.players.get_mut(entity).unwrap();
                object.velocity = (object.velocity.0 * factor, object.velocity.1 * factor);
            },
            Action::SpawnCrumble(g) => {
                let enemy = game.enemies.get_mut(*g).unwrap().1.get_mut(entity).unwrap();
                let (x, y, v, r) = (enemy.x, enemy.y, enemy.velocity.clone(), enemy.radius / 2.0);
                // cumble
                let mut crumble = Enemy::new(x, y, vector::normalize(v, 0.5), r, "rgb(0,0,0)");
                crumble.effects.push(EnemyEffect::Lifetime(2000));
                game.enemies.get_mut(*g).unwrap().1.push(crumble);
            },
            Action::Despawn(g) => {
                game.enemies.get_mut(*g).unwrap().1.remove(entity);
            },
            Action::ReduceLifetime(g) => {
                // TODO why does unwrap throw an error?
                // println!("{} < {}", entity, game.enemies.get_mut(*g).unwrap().1.len());
                // let enemy = game.enemies.get_mut(*g).unwrap().1.get_mut(entity).unwrap();
                let enemy = match game.enemies.get_mut(*g).unwrap().1.get_mut(entity) {
                    Some(e) => e,
                    None => {
                        // println!("tried to reduce lifetime of {entity} ({})", game.enemies.get_mut(*g).unwrap().1.len());
                        return;
                    },
                };
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::EnemyEffect::Lifetime(t) => {
                            if *t == 0 {
                                game.enemies.get_mut(*g).unwrap().1.remove(entity);
                                break;
                            }
                            else {
                                *t -= 1;
                            }
                        },
                        _ => {},
                    }
                }
            },
            Action::ReduceCooldown(group) => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::EnemyEffect::Shoot { radius, speed, time_left, cooldown, lifetime, projectile_radius, color } => {
                            if *time_left > 0 {
                                *time_left -= 1;
                            }
                        },
                        crate::enemy::EnemyEffect::Explode { lifetime, radius, speed, amount, time_left, cooldown, color } => {
                            if *time_left > 0 {
                                *time_left -= 1;
                            }
                        },
                        _ => {},
                    }
                }
            },
            Action::ResetCooldown(group) => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                for effect in enemy.effects.iter_mut() {
                    match effect {
                        crate::enemy::EnemyEffect::Shoot { radius, speed, time_left, cooldown, lifetime, projectile_radius, color } => {
                            *time_left = *cooldown;
                        },
                        crate::enemy::EnemyEffect::Explode { lifetime, radius, speed, amount, time_left, cooldown, color } => {
                            *time_left = *cooldown;
                        },
                        _ => {},
                    }
                }
            },
            Action::SpawnProjectile { group, velocity, color, radius, lifetime } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                // projectile
                let mut crumble = Enemy::new(enemy.x, enemy.y, *velocity, *radius, color.as_str());
                crumble.effects.push(EnemyEffect::Lifetime(*lifetime));
                game.enemies.get_mut(*group).unwrap().1.push(crumble);
            },
        }
    }
}

