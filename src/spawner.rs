use rand::Rng;

use crate::{enemy::{Enemy, EnemyEffect}, game::{DrawPack, Game, Shape}, gametraits::Radius, wall::WallType};


impl Game {
    pub fn spawn_dirt_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Dirt, WallType::SpawnA];
        let mut enemies = vec![];
        for _ in 0..120 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(1500.0, 1000.0, velocity, rand::thread_rng().gen_range(10.0..=50.0), "rgb(50,40,20)");
            enemy.effects.push(EnemyEffect::Crumble);
            enemy.effects.push(EnemyEffect::ShrinkPlayers { radius: Radius::Relative(10.0), shrink: 0.9, duration: 1 });
            enemy.draw_packs.push(DrawPack::new("rgba(50,40,20,0.2)", Shape::Circle { radius: Radius::Relative(10.0) }, (0.0, 0.0)));
            enemy.view_radius = Radius::Relative(10.0);
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_wind_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Wind, WallType::SpawnA];
        let mut enemies = vec![];
        for _ in 0..40 * spawn_m {
            let cap = 0.8 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-1000.0, 1000.0, velocity, rand::thread_rng().gen_range(40.0..=100.0), "rgb(200,200,255)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,255,255,0.1)", Shape::Circle { radius: Radius::Relative(3.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(3.0), power: 5.0 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_flower_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Flower, WallType::SpawnA];
        let mut enemies = vec![];
        for _ in 0..150 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-1000.0, -1000.0, velocity, rand::thread_rng().gen_range(10.0..=30.0), "rgb(255,250,5)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,255,0.2)", Shape::Circle { radius: Radius::Relative(5.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Chase { radius: Radius::Relative(5.0), power: 0.2});
            // enemy.harmless = true;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_water_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Water, WallType::SpawnA];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(2000.0, -2000.0, velocity, rand::thread_rng().gen_range(50.0..=100.0), "rgb(50,50,200)");
            enemies.push(enemy);
        }
        for _ in 0..5 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let radius = rand::thread_rng().gen_range(400.0..=600.0);
            let mut enemy = Enemy::new(3000.0, -3000.0, velocity, radius, "rgb(10,10,100)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(10,10,100,0.5)", Shape::Circle { radius: Radius::Relative(1.3) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(1.3), power: -2.0 });
            // enemy.draw_packs.push(DrawPack::new("", Shape::Image { keyword: "candytop".to_owned(), scale: radius / 300.0 }, (-radius / 1.2, -radius / 1.2)));
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_fire_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let size = 20.0..=50.0;
        let amount = 150;
        let speed = 1.0;
        let dist = 4500.0;
        let ids = vec![WallType::Dirt,WallType::Wind,WallType::Flower,WallType::Water,WallType::Fire,WallType::SpawnA,WallType::SpawnB];
        let mut enemies = vec![];
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(0.0, -dist, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(0.0, dist, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(-dist, 0.0, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let enemy = Enemy::new(dist, 0.0, velocity, rand::thread_rng().gen_range(size.clone()), "red");
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_blackhole_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let size = 200.0..=500.0;
        let amount = 50;
        let speed = 2.0;
        let dist = 15000.0;
        let ids = vec![
            WallType::Fire,
            WallType::Shooting,
            WallType::Explosion,
            WallType::Snake,
            WallType::Ice,
            WallType::Blackhole,
            WallType::SpawnB,
            WallType::Poison,
            WallType::Hell,
            WallType::Candy,
            WallType::Lightning,
        ];
        let color = "black";
        let auracolor = "rgba(0,0,0,0.2)";
        let mut enemies = vec![];
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(dist, -dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.view_radius = Radius::Relative(2.0);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(dist, dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.view_radius = Radius::Relative(2.0);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-dist, -dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.view_radius = Radius::Relative(2.0);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        for _ in 0..amount * spawn_m {
            let cap = speed * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-dist, dist, velocity, rand::thread_rng().gen_range(size.clone()), color);
            enemy.view_radius = Radius::Relative(2.0);
            enemy.draw_packs.push(DrawPack::new(auracolor, Shape::Circle { radius: Radius::Relative(2.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(2.0), power: -6.0 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_tech_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Shooting];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.2 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-20000.0, 0.0, velocity, rand::thread_rng().gen_range(30.0..=30.0), "rgb(25,25,25)");
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,255,0,0.02)", Shape::Circle { radius: Radius::Relative(30.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Shoot { radius: Radius::Relative(30.0), speed: 10.0, cooldown: 60, time_left: 0, lifetime: 1000, projectile_radius: 20.0, color: "black".to_owned(), effects: vec![], under_dps: vec![] });
            enemy.view_radius = Radius::Relative(30.0);
            enemies.push(enemy);
        }
        for _ in 0..50 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(-20000.0, 0.0, velocity, rand::thread_rng().gen_range(30.0..=70.0), "rgb(255,125,125)");
            let r = Radius::Relative(5.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,0,0.1)", Shape::Circle { radius: r }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: r, slow: 0.5, duration: 1 });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_ice_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Ice];
        let mut enemies = vec![];
        // snowballs
        for _ in 0..25 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(0.0, -20000.0, velocity, rand::thread_rng().gen_range(50.0..=70.0), "rgb(255,255,255)");
            let r = Radius::Relative(2.0);
            enemy.effects.push(EnemyEffect::Grow { size: 0.2, maxsize: 10.0 * enemy.radius, defaultsize: enemy.radius });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        // snowmans
        for _ in 0..25 * spawn_m {
            let cap = 0.5 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(0.0, -20000.0, velocity, rand::thread_rng().gen_range(50.0..=70.0), "rgb(255,255,255)");
            enemy.draw_packs.push(DrawPack::new("rgb(255,150,0)", Shape::Circle { radius: Radius::Relative(0.8)}, (0.0, 0.0)));
            enemy.draw_packs.push(DrawPack::new("rgb(0,0,0)", Shape::Circle { radius: Radius::Relative(0.1)}, (-10.0, -10.0)));
            enemy.draw_packs.push(DrawPack::new("rgb(0,0,0)", Shape::Circle { radius: Radius::Relative(0.1)}, (10.0, -10.0)));
            let r = Radius::Relative(10.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(0,0,255,0.05)", Shape::Circle { radius: Radius::Relative(10.0) }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Chase { radius: r, power: 0.03 });
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: r, slow: 0.8, duration: 1 });
            enemy.effects.push(EnemyEffect::Shoot { lifetime: 200, radius: r, projectile_radius: 20.0, speed: 8.0, time_left: 0, cooldown: 100, color: "rgb(200,200,200)".to_owned(), effects: vec![], under_dps: vec![] });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_snake_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Snake];
        let mut enemies = vec![];
        for _ in 0..50 * spawn_m {
            let cap = 0.8 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(20000.0, 0.0, velocity, 90.0, "rgb(25,25,25)");
            let r = Radius::Relative(20.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(0,255,255,0.02)", Shape::Circle { radius: r }, (0.0, 0.0)));
            enemy.effects.push(EnemyEffect::Shoot { radius: r, speed: 10.0, cooldown: 5, time_left: 0, lifetime: 50, projectile_radius: 40.0, color: "rgb(0,0,50)".to_owned(), effects: vec![], under_dps: vec![] });
            enemy.view_radius = r;
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_explosion_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Explosion];
        let mut enemies = vec![];
        for _ in 0..20 * spawn_m {
            let cap = 0.1 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let mut enemy = Enemy::new(0.0, 20000.0, velocity, 90.0, "rgb(25,25,25)");
            let cd = rand::thread_rng().gen_range(200..=400);
            let radius = rand::thread_rng().gen_range(10.0..=30.0);
            enemy.effects.push(EnemyEffect::Explode { lifetime: 400, radius: (10.0, 30.0), speed: 10.0, time_left: 0, cooldown: cd, color: "rgb(255,255,0)".to_owned(), amount: 10, effects: Vec::new(), under_dps: vec![] });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_lightning_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Lightning];
        let mut enemies = vec![];
        for _ in 0..20 * spawn_m {
            let cap = 0.1 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let cloudradius = rand::thread_rng().gen_range(100.0..=300.0);
            let color = "rgba(100,80,150,0.7)";
            let mut enemy = Enemy::new(-25000.0, 25000.0, velocity, cloudradius, color);
            enemy.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: Radius::Relative(0.8) }, (cloudradius, cloudradius / 5.0)));
            enemy.draw_packs.push(DrawPack::new(color, Shape::Circle { radius: Radius::Relative(0.7) }, (-cloudradius, cloudradius / 4.0)));
            enemy.harmless = true;
            let cd = rand::thread_rng().gen_range(400..=500);
            let lightning_aura_radius = Radius::Relative(5.0);
            enemy.effects.push(EnemyEffect::Explode {
                lifetime: 400,
                radius: (10.0, 30.0),
                speed: 15.0,
                time_left: 0,
                cooldown: cd,
                color: "rgb(255,255,255)".to_owned(),
                amount: (cloudradius / 20.0) as usize,
                effects: vec![EnemyEffect::SlowPlayers { radius: lightning_aura_radius, slow: 0.0, duration: 100 }],
                under_dps: vec![DrawPack::new("rgba(255,255,0,0.2)", Shape::Circle { radius: lightning_aura_radius }, (0.0,0.0))],
            });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_hell_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Hell];
        let mut enemies = vec![];
        for _ in 0..20 * spawn_m {
            let cap = 0.8 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let radius = rand::thread_rng().gen_range(100.0..=300.0);
            let color = "rgb(60,0,0)";
            let mut enemy = Enemy::new(25000.0, -25000.0, velocity, radius, color);
            let cd = rand::thread_rng().gen_range(400..=500);
            let fire_aura_radius = Radius::Relative(5.0);
            enemy.draw_packs.insert(0, DrawPack::new("rgba(255,0,0,0.2)", Shape::Circle { radius: Radius::Relative(3.0) }, (0.0, 0.0)));
            enemy.view_radius = Radius::Relative(3.0);
            enemy.effects.push(EnemyEffect::Explode {
                lifetime: 400,
                radius: (10.0, 30.0),
                speed: 15.0,
                time_left: 0,
                cooldown: cd,
                color: "rgb(255,255,0)".to_owned(),
                amount: (radius / 20.0) as usize,
                effects: vec![EnemyEffect::SlowPlayers { radius: fire_aura_radius, slow: 0.3, duration: 100 }, EnemyEffect::Chase { radius: Radius::Absolute(1000.0), power: 0.2 }],
                under_dps: vec![DrawPack::new("rgba(255,200,0,0.2)", Shape::Circle { radius: fire_aura_radius }, (0.0,0.0))],
            });
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: Radius::Relative(3.0), slow: 0.5, duration: 1 });
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_poison_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Poison];
        let mut enemies = vec![];
        for _ in 0..150 * spawn_m {
            let cap = 0.6 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let radius = rand::thread_rng().gen_range(100.0..=200.0);
            let color = "rgb(0,255,0)";
            let mut enemy = Enemy::new(25000.0, 25000.0, velocity, radius, color);
            enemy.effects.push(EnemyEffect::SlowPlayers { radius: Radius::Relative(3.0), slow: 0.5, duration: 200 });
            enemy.draw_packs.push(DrawPack::new("rgba(0,255,0,0.2)", Shape::Circle { radius: Radius::Relative(3.0) }, (0.0, 0.0)));
            enemy.view_radius = Radius::Relative(3.0);
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }

    pub fn spawn_candy_enemies(&mut self, speed_m: f32, spawn_m: i32) {
        let ids = vec![WallType::Candy];
        let mut enemies = vec![];
        for _ in 0..400 * spawn_m {
            let cap = 0.3 * speed_m;
            let velocity: (f32, f32) = (rand::thread_rng().gen_range(-cap..=cap), rand::thread_rng().gen_range(-cap..=cap));
            let radius = rand::thread_rng().gen_range(50.0..=100.0);
            let color = "rgb(255,0,255)";
            let mut enemy = Enemy::new(-25000.0, -25000.0, velocity, radius, color);
            enemy.id = 1;
            enemy.effects.push(EnemyEffect::Chase { radius: Radius::Relative(3.0), power: -0.05 });
            enemy.effects.push(EnemyEffect::Push { radius: Radius::Relative(4.0), power: -1.5 });
            enemy.draw_packs.push(DrawPack::new("rgba(255,0,255,0.2)", Shape::Circle { radius: Radius::Relative(4.0) }, (0.0, 0.0)));
            enemy.draw_packs.push(DrawPack::new("", Shape::Image { keyword: "candytop".to_owned(), scale: radius / 300.0 }, (-radius / 1.2, -radius / 1.2)));
            enemy.view_radius = Radius::Relative(4.0);
            enemies.push(enemy);
        }
        self.enemies.push((ids, enemies)); 
    }
}
