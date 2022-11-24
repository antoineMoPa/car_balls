use std::ops::Mul;

use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

#[derive(Default)]
struct Game {
    player_car: Option<Entity>,
    camera_follow: Option<Vec3>,
    camera: Option<Entity>,
}

fn main() {
    App::new()
        .init_resource::<Game>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_dynamic_objects)
        .add_system(print_ball_altitude)
        .add_system(keyboard_input_system)
        .add_system(camera_follow_car_system)
        .add_system(camera_follow_target_system)
        .run();
}

fn setup_graphics(
    mut commands: Commands,
    mut game: ResMut<Game>
) {
    game.camera = Some(
        commands.spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        }).id());
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut transforms: Query<&mut Transform>,
    game: ResMut<Game>,
    mut ext_forces: Query<&mut ExternalForce>,
) {
    let entity = match game.player_car {
        Some(entity) => entity,
        _ => {
            return;
        }
    };
    let mut ext_force = match ext_forces.get_mut(entity) {
        Ok(ext_force) => ext_force,
        _ => {
            return;
        }
    };
    let transform = match transforms.get_mut(entity) {
        Ok(transform) => transform,
        _ => {
            return;
        }
    };

    // Apply forces
    let forward_speed: f32 = 100.0;
    let backward_speed: f32 = -40.0;
    if keyboard_input.pressed(KeyCode::W) {
        ext_force.force = transform.forward().mul(Vec3 { x: forward_speed, y: forward_speed, z: forward_speed });
        ext_force.torque = Vec3::new(0.0, 0.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::S) {
        ext_force.force = transform.forward().mul(Vec3 { x: backward_speed, y: backward_speed, z: backward_speed });
        ext_force.torque = Vec3::new(0.0, 0.0, 0.0);
    }

    let torque: f32 = 12.0;

    if keyboard_input.pressed(KeyCode::A) {
        ext_force.force = Vec3::new(0.0, 0.0, 0.0);
        ext_force.torque = Vec3::new(0.0, torque, 0.0);
    }

    if keyboard_input.pressed(KeyCode::D) {
        ext_force.force = Vec3::new(0.0, 0.0, 0.0);
        ext_force.torque = Vec3::new(0.0, -torque, 0.0);
    }
}


fn camera_follow_car_system(
    mut transforms: Query<&mut Transform>,
    mut game: ResMut<Game>,
) {
    let car_entity = match game.player_car {
        Some(entity) => entity,
        _ => {
            return;
        }
    };
    let car_transform = match transforms.get_mut(car_entity) {
        Ok(transform) => transform,
        _ => {
            return;
        }
    };
    game.camera_follow = Some(car_transform.translation);
}

fn camera_follow_target_system(
    mut transforms: Query<&mut Transform>,
    game: ResMut<Game>,
) {
    let camera_entity = match game.camera {
        Some(entity) => entity,
        _ => {
            return;
        }
    };
    let mut camera_transform = match transforms.get_mut(camera_entity) {
        Ok(camera_transform) => camera_transform,
        _ => {
            return;
        }
    };
    match game.camera_follow {
        Some(camera_follow) => camera_transform.look_at(camera_follow, Vec3::Y),
        _ => {
            return;
        }
    }
}

fn setup_dynamic_objects(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    /* Create the ground. */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    /* Create the bouncing ball. */
    let my_gltf = asset_server.load("model.glb#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle

    game.player_car =
        Some(
            commands
                .spawn_bundle(SceneBundle {
                    scene: my_gltf,
                    transform: Transform::from_xyz(2.0, 0.0, -5.0),
                    ..Default::default()
                })
                .insert(RigidBody::Dynamic)
                .insert(Collider::cuboid(1.0, 1.0, 4.0))
                .insert(ColliderMassProperties::Density(0.04))
                .insert(Friction::coefficient(0.0))
                .insert(Damping { linear_damping: 0.8, angular_damping: 0.4 })
                .insert(ExternalForce {
                    force: Vec3::new(0.0, 0.0, 0.0),
                    torque: Vec3::new(0.0, 0.0, 0.0),
                })
                .id());

    for i in 1..15 {
        for j in 1..15 {
            commands
                .spawn()
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(0.5))
                .insert(Restitution::coefficient(0.7))
                .insert(ColliderMassProperties::Density(0.02))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(i as f32 * 2.0, 4.0, j as f32 * 2.0)));
        }
    }
}

fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        println!("Ball altitude: {}", transform.translation.y);
    }
}
