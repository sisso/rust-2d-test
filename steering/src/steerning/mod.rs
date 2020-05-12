pub mod components;
mod systems;

use components::*;
use systems::*;

use commons::math::*;

use cgmath::{prelude::*, Deg, Point2, Vector2};
use ggez::graphics::Color;
use ggez::{GameError, GameResult};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use specs::prelude::*;
use specs::{World, WorldExt};

// #[derive(Debug, Clone)]
// enum Error {
//     Other(String),
// }
//
// type Result<T> = std::result::Result<T, Error>;

pub fn load_cfg() -> GameResult<Cfg> {
    let reader =
        std::io::BufReader::new(std::fs::File::open("steering/resources/config.json").unwrap());

    let cfg: serde_json::Result<Cfg> = serde_json::from_reader(reader);

    match cfg {
        Ok(cfg) => Ok(cfg),
        Err(e) => Err(GameError::ConfigError("invalid config file".to_string())),
    }
}

pub fn create_world(cfg: Cfg) -> GameResult<World> {
    // create world
    let mut world = World::new();
    world.register::<Model>();
    world.register::<Cfg>();
    world.register::<Vehicle>();
    world.register::<MovingArea>();
    world.register::<SteeringArrival>();
    world.register::<SteeringSeparation>();
    world.register::<SteeringVelocity>();
    world.register::<Wall>();
    world.register::<SteeringFormationMember>();
    world.register::<SteeringFormationLeader>();

    world.insert(GameTime { delta_time: 0.01 });
    world.insert(DebugStuff::new());
    world.insert(cfg);

    Ok(world)
}

pub fn initialize_world(world: &mut World) -> GameResult<()> {
    let cfg = {
        let cfg: &Cfg = &world.read_resource::<Cfg>();
        cfg.clone()
    };

    let mut rng: StdRng = SeedableRng::seed_from_u64(cfg.seed);

    {
        let (width, height) = (cfg.screen_width, cfg.screen_height);
        let border = 80.0;
        let points: Vec<P2> = vec![
            (border, border).into(),
            (width - border, border).into(),
            (width - border, height - border).into(),
            (border, height - border).into(),
        ];

        // world.insert(MovingArea {
        //     polygons: vec![Polygon::try_new(points.clone()).unwrap()],
        // });

        let wall_width = 15.0;
        let wall_force = 200.0;

        for (p0, p1, distance, force) in vec![
            (
                points[0],
                points[1],
                wall_width,
                Vector2::new(0.0, 1.0) * wall_force,
            ),
            (
                points[1],
                points[2],
                wall_width,
                Vector2::new(-1.0, 0.0) * wall_force,
            ),
            (
                points[2],
                points[3],
                wall_width,
                Vector2::new(0.0, -1.0) * wall_force,
            ),
            (
                points[3],
                points[0],
                wall_width,
                Vector2::new(1.0, 0.0) * wall_force,
            ),
        ] {
            let p0 = Point2::new(p0.x as f32, p0.y as f32);
            let p1 = Point2::new(p1.x as f32, p1.y as f32);

            world
                .create_entity()
                .with(Wall::new_from_points(p0, p1, distance, force))
                .build();
        }
    }

    let max_acc = cfg.max_acc;
    let max_speed = cfg.max_speed;
    let separation_mut = cfg.separation_radius;

    let mut formation_index = 0;

    for i in 0..cfg.vehicles {
        let pos = Point2::new(
            rng.gen_range(cfg.start_position[0], cfg.start_position[2]),
            rng.gen_range(cfg.start_position[1], cfg.start_position[3]),
        );

        let vel = Vector2::new(
            rng.gen_range(-max_speed, max_speed),
            rng.gen_range(-max_speed, max_speed),
        );

        let radius = if rng.gen::<f32>() <= 0.1f32 { 6.0 } else { 3.0 };
        let follow = i < cfg.followers;

        let color = if follow {
            if formation_index == 0 {
                Color::new(0.0, 1.0, 1.0, 1.0)
            } else {
                Color::new(1.0, 1.0, 0.0, 1.0)
            }
        } else {
            Color::new(1.0, 0.0, 0.0, 1.0)
        };

        let dir = rotate_vector_by_angle(Vector2::unit_y(), Deg(rng.gen_range(0.0, 360.0)).into());

        let mut builder = world
            .create_entity()
            .with(Vehicle {
                pos,
                dir: dir,
                desired_dir: dir,
                vel_dir: Vector2::zero(),
                speed: 0.0,
                max_acc: max_acc,
                rotation_speed: deg2rad(cfg.rotation_speed),
                desired_vel: Vector2::zero(),
                max_speed,
            })
            .with(Model {
                size: radius,
                pos: Point2::new(0.0, 0.0),
                dir,
                color,
            })
            .with(SteeringArrival {
                enabled: follow,
                target_pos: Point2::new(300.0, 300.0),
                distance: cfg.arrival_distance,
                weight: 1.0,
                arrived: false,
            })
            .with(SteeringSeparation {
                enabled: true,
                distance: radius * separation_mut,
                weight: 2.0,
            })
            .with(SteeringVelocity {
                enabled: !follow,
                vel: vel,
                weight: 1.0,
            });

        if follow {
            if formation_index == 0 {
                builder = builder.with(SteeringFormationLeader {
                    formation: FormationType::Line,
                });
            }

            builder = builder.with(SteeringFormationMember {
                index: formation_index,
            });

            formation_index += 1;
        }

        builder.build();
    }

    Ok(())
}

pub fn run(delta: f32, world: &mut World) {
    world.insert(GameTime { delta_time: delta });

    let mut dispatcher = DispatcherBuilder::new()
        .with(SteerArrivalSystem, "steering_arrival", &[])
        .with(SteeringSeparationSystem, "steering_separation", &[])
        .with(SteeringVelocitySystem, "steering_velocity", &[])
        .with(SteeringWallsSystem, "steering_walls", &[])
        // .with(SteeringFormationSystem, "steering_formation", &[])
        .with(
            MoveSystem,
            "move",
            &[
                "steering_arrival",
                "steering_separation",
                "steering_velocity",
                "steering_walls",
                // "steering_formation",
            ],
        )
        .with(BordersTeleportSystem, "border_teleport", &["move"])
        .with(UpdateModelPosSystem, "update_model", &["border_teleport"])
        .build();

    dispatcher.run_now(world);
}

pub fn move_to(world: &mut World, target_pos: P2) -> GameResult<()> {
    let mut arrivals = world.write_component::<SteeringArrival>();
    let formations = world.read_component::<SteeringFormationMember>();
    let leaders = world.read_component::<SteeringFormationLeader>();
    let vehicles = world.read_component::<Vehicle>();

    let mut total = formations.count();
    let mut formation: Option<(P2, FormationType)> = None;

    // get leader position
    for (leader, vehicle) in (&leaders, &vehicles).join() {
        let pos = vehicle.pos;
        formation = Some((pos, leader.formation));
    }

    let (leader_pos, leader_formation) = formation.unwrap();

    let dir = (target_pos - leader_pos).normalize();

    // update target positions
    for (formation, arrival) in (&formations, &mut arrivals).join() {
        let index = formation.index;
        let pos = leader_formation.get_pos(dir, target_pos, total, index);
        arrival.target_pos = pos;
    }

    Ok(())
}

pub fn take_debug_lines(world: &mut World) -> Vec<(P2, P2, Color)> {
    let stuff = &mut world.write_resource::<DebugStuff>();
    stuff.take_lines()
}
