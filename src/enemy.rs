use rand::Rng;

use crate::{action::{Action}, game::{DrawPack, Game, Shape}, impl_Drawable, impl_Movable, impl_Position, vector};
use crate::gametraits::*;

#[derive(Default)]
pub struct Enemy {
    pub id: usize,
    pub velocity: (f32, f32),
    pub speed_multiplier: f32,
    pub x: f32,
    pub y: f32,
    pub draw_packs: Vec<DrawPack>,
    pub radius: f32,
    pub effects: Vec<EnemyEffect>,
    pub just_collided: bool,
    pub view_radius: f32,
    pub harmless: bool,
}

impl_Position!(Enemy);
impl_Movable!(Enemy);
impl_Drawable!(Enemy);

impl Enemy {
    pub fn new(x: f32, y: f32, velocity: (f32, f32), radius: f32, color: &str) -> Enemy {
        let mut p = Enemy {
            x,y,
            velocity,
            radius,
            view_radius: radius,
            draw_packs: vec![],
            speed_multiplier: 1.0,
            ..Default::default()
        };
        p.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: p.radius }, (0.0, 0.0)));

        p
    }
}

#[derive(Clone)]
pub enum EnemyEffect {
    Chase {radius: f32, power: f32},
    Crumble,
    Lifetime(usize),
    Push {radius: f32, power: f32},
    Shoot {lifetime: usize, radius: f32, projectile_radius: f32, speed: f32, time_left: usize, cooldown: usize, color: String, effects: Vec<EnemyEffect>},
    Explode {lifetime: usize, radius: f32, speed: f32, amount: usize, time_left: usize, cooldown: usize, color: String, effects: Vec<EnemyEffect>, underDPs: Vec<DrawPack>},
    SlowPlayers {radius: f32, slow: f32, duration: usize},
    Grow {size: f32, maxsize: f32, defaultsize: f32},
    SpeedAlter {origin: usize, slow: f32, ease: usize},
}

pub fn handle_effects(game: &mut Game) {
    let mut actions: Vec<(usize, Action)> = vec![];
    for (g, group) in game.enemies.iter().enumerate() {
        for (i, enemy) in group.1.iter().enumerate() {
            for (e, effect) in enemy.effects.iter().enumerate() {
                match effect {
                    EnemyEffect::Chase { radius, power } => {
                        for player in game.players.iter() {
                            // if !player.alive {continue;}
                            let dist = vector::distance((enemy.x, enemy.y), (player.x, player.y));
                            if dist.2 <= *radius + player.radius {
                                let add = vector::normalize((dist.0, dist.1), *power);
                                actions.push((i, Action::UpdateEnemyVelocity(g, (enemy.velocity.0 + add.0, enemy.velocity.1 + add.1))));
                            }
                        }
                    }
                    EnemyEffect::Crumble => {
                        if enemy.just_collided {
                            actions.push((i, Action::SpawnCrumble(g)));
                        }
                    },
                    EnemyEffect::Lifetime(t) => {
                        actions.push((i, Action::ReduceLifetime { group: g, effect: e }));
                    },
                    EnemyEffect::Push { radius, power } => {
                        for (p, player) in game.players.iter().enumerate() {
                            let dist = vector::distance((enemy.x, enemy.y), (player.x, player.y));
                            if dist.2 <= *radius + player.radius {
                                let add = vector::normalize((dist.0, dist.1), *power);
                                actions.push((p, Action::AddPlayerVelocity(add)));
                            }
                        }
                    },
                    EnemyEffect::Shoot { radius, speed, cooldown, time_left, lifetime, projectile_radius, color, effects } => {
                        for player in game.players.iter() {
                            if !player.alive {continue;}
                            let dist = vector::distance((enemy.x, enemy.y), (player.x, player.y));
                            if dist.2 <= *radius + player.radius {
                                let v = vector::normalize((dist.0, dist.1), *speed);
                                if *time_left == 0 {
                                    actions.push((i, Action::SpawnProjectile { group: g, velocity: v, radius: *projectile_radius, color: color.clone(), lifetime: *lifetime, effects: effects.clone(), underDPs: vec![] }));
                                    actions.push((i, Action::ResetCooldown(g)));
                                }
                                break;
                            }
                        }
                        actions.push((i, Action::ReduceCooldown(g)));
                    },
                    EnemyEffect::Explode { lifetime, radius, speed, amount, time_left, cooldown, color, effects, underDPs } => {
                        if *time_left == 0 {
                            for _ in 0..*amount {
                                let v = (rand::thread_rng().gen_range(-*speed..=*speed), rand::thread_rng().gen_range(-*speed..=*speed));
                                actions.push((i, Action::SpawnProjectile { group: g, velocity: v, radius: *radius, color: color.clone(), lifetime: *lifetime, effects: effects.clone(), underDPs: underDPs.clone() }));
                            }
                            actions.push((i, Action::ResetCooldown(g)));
                        }
                        actions.push((i, Action::ReduceCooldown(g)));
                    },
                    EnemyEffect::SlowPlayers { radius, slow, duration } => {
                        for (p, player) in game.players.iter().enumerate() {
                            if !player.alive {continue;}
                            let dist = vector::distance((enemy.x, enemy.y), (player.x, player.y));
                            if dist.2 <= *radius + player.radius {
                                let id = enemy.id;
                                // actions.push((p, Action::MulPlayerVelocity(*power)));
                                // check if effect of this item id is already applied
                                let position = enemy.effects.iter().position(|e| {
                                    match e {
                                        crate::enemy::EnemyEffect::SpeedAlter { origin, slow, ease } => {
                                            *origin == id
                                        },
                                        _ => {
                                            false
                                        }
                                    }
                                });
                                match position {
                                    Some(e) => {
                                        let effect = enemy.effects.get(e).unwrap();
                                        match effect {
                                            crate::enemy::EnemyEffect::SpeedAlter { origin, slow, ease } => {
                                                // ease = *duration;
                                                actions.push((e, Action::SetEnemySpeedAlterEase { group: g, effect: e, value: *duration }));
                                            },
                                            _ => { }
                                        };
                                    },
                                    None => {
                                        let effect = crate::player::PlayerEffect::SpeedAlter { slow: *slow, ease: *duration, origin: id };
                                        actions.push((p, Action::PushPlayerEffect(effect)));
                                    },
                                }
                            }
                        }
                    },
                    EnemyEffect::Grow { size, maxsize, defaultsize } => {
                        if enemy.just_collided {
                            actions.push((i, Action::SetEnemyRadius(g, *defaultsize)));
                        }
                        else if enemy.radius < *maxsize {
                            actions.push((i, Action::SetEnemyRadius(g, enemy.radius + *size)));
                        }
                    },
                    EnemyEffect::SpeedAlter { slow, ease, origin } => {
                        if *ease == 0 {
                            // remove this effect
                            actions.push((i, Action::RemoveEnemyEffect { group: g, effect: e }));
                        }
                        else {
                            actions.push((i, Action::DecrementEnemySpeedAlterEase { group: g, effect: e }));
                            actions.push((i, Action::MulEnemySpeedMultiplier { group: g, f: *slow }));
                        }
                    },
                }
            }
        }
    }
    // reset enemy speed multiplier to 1.0
    for group in game.enemies.iter_mut() {
        for enemy in group.1.iter_mut() {
            enemy.speed_multiplier = 1.0;
        }
    }
    // reverse order due to deletions and index errors
    for (entity, action) in actions.iter().rev() {
        action.execute(game, *entity);
    }
}
