
use crate::{enemy::{Enemy, EnemyEffect}, game::Game, player::PlayerEffect, vector};

pub enum Action {
    UpdateEnemyVelocity(usize, (f32,f32)),
    SetPlayerVelocity((f32,f32)),
    SetPlayerZoomlimit((f32,f32)),
    AddPlayerVelocity((f32,f32)),
    MulPlayerVelocity(f32),
    SpawnCrumble(usize),
    ReduceLifetime{group: usize, effect: usize},
    ReduceCooldown(usize),
    ResetCooldown(usize),
    Despawn(usize),
    SpawnProjectile { group: usize, velocity: (f32, f32), radius: f32, color: String, lifetime: usize },
    SetEnemyRadius(usize, f32),
    SetPlayerSpeed(f32),
    MulPlayerSpeed(f32),
    RemoveEnemyEffect {group: usize, effect: usize},
    RemovePlayerEffect {effect: usize},
    MulEnemySpeedMultiplier {group: usize, f: f32},
    MulPlayerSpeedMultiplier {f: f32},
    PushPlayerEffect(PlayerEffect),
    DecrementEnemySpeedAlterEase{group: usize, effect: usize},
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
            Action::SetPlayerZoomlimit(v) => {
                let object = game.players.get_mut(entity).unwrap();
                object.zoomlimit = *v;
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
            Action::ReduceLifetime { group, effect } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                let effect = enemy.effects.get_mut(*effect).unwrap();
                match effect {
                    EnemyEffect::Lifetime(t) => {
                        if *t == 0 {
                            game.enemies.get_mut(*group).unwrap().1.remove(entity);
                        }
                        else {
                            *t -= 1;
                        }
                    },
                    _ => {},
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
            Action::SetEnemyRadius(group, radius) => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                enemy.radius = *radius;
                enemy.view_radius = *radius;
                for dp in enemy.draw_packs.iter_mut().rev() {
                    match &mut dp.shape {
                        crate::game::Shape::Circle { radius: r } => {
                            *r = *radius;
                            break;
                        },
                        _ => {}
                    }
                }
            },
            Action::SetPlayerSpeed(s) => {
                let player = game.players.get_mut(entity).unwrap();
                player.speed = *s;
            },
            Action::MulPlayerSpeed(s) => {
                let player = game.players.get_mut(entity).unwrap();
                player.speed *= *s;
            },
            Action::RemoveEnemyEffect { group, effect } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                enemy.effects.remove(*effect);
            },
            Action::MulEnemySpeedMultiplier { group, f } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                enemy.speed_multiplier *= *f;
            },
            Action::MulPlayerSpeedMultiplier { f } => {
                let player = game.players.get_mut(entity).unwrap();
                player.speed_multiplier *= *f;
            },
            Action::RemovePlayerEffect { effect } => {
                let player = game.players.get_mut(entity).unwrap();
                player.effects.remove(*effect);
            },
            Action::PushPlayerEffect(effect) => {
                let player = game.players.get_mut(entity).unwrap();
                player.effects.push(*effect);
            },
            Action::DecrementEnemySpeedAlterEase { group, effect } => {
                let enemy = game.enemies.get_mut(*group).unwrap().1.get_mut(entity).unwrap();
                let effect = enemy.effects.get_mut(*effect).unwrap();
                match effect {
                    EnemyEffect::SpeedAlter { origin, slow, ease } => {
                        *ease -= 1;
                    },
                    _ => { }
                }
            },
        }
    }
}

