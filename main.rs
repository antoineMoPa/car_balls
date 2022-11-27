use std::{ops::Mul};
use bevy::{
    input::{keyboard::KeyCode, Input},
    pbr::DirectionalLightShadowMap,
    prelude::*,
};
use bevy_rapier3d::prelude::*;

#[derive(Default)]
struct CameraTarget {
    position: Option<Vec3>,
    up: Option<Vec3>,
    look_at: Option<Vec3>,
}

#[derive(Default)]
struct Game {
    player_car: Option<Entity>,
    camera_target: CameraTarget,
    camera: Option<Entity>,
}


fn main() {
    App::new()
        .init_resource::<Game>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(DirectionalLightShadowMap { size: 2048 })
        .insert_resource(AmbientLight {
            color: Color::rgb(0.6, 0.4, 0.5),
            brightness: 0.6,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_dynamic_objects)
        .add_startup_system(setup_window_size)
        .add_system(print_ball_altitude)
        .add_system(keyboard_input_system)
        .add_system(camera_target_car_system)
        .add_system(camera_target_target_system)
        .run();
}

fn setup_window_size(mut windows: ResMut<Windows>) {
    let window = match windows.get_primary_mut() {
        Some(window) => window,
        _ => {
            return;
        }
    };
    let wasm_window = match web_sys::window() {
        Some(wasm_window) => wasm_window,
        _ => {
            return;
        }
    };
    let (target_width, target_height) = (
        wasm_window.inner_width().unwrap().as_f64().unwrap() as f32,
        wasm_window.inner_height().unwrap().as_f64().unwrap() as f32,
    );

    window.set_resolution(target_width, target_height);
}

#[cfg(not(target_arch = "wasm32"))]
fn setup_window_size() {
}

fn setup_graphics(
    mut commands: Commands,
    mut game: ResMut<Game>,
) {
    game.camera = Some(
        commands.spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        }).id());

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.0,
            color: Color::rgb(0.5, 0.5, 2.0),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_xyzw(-1.0, -0.3, 0.0, 0.0),
            ..default()
        },
        ..default()
    });
}

fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
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

    ext_force.force = Vec3::ZERO;
    ext_force.torque = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::Up) {
        ext_force.force = transform.forward().mul(Vec3 { x: forward_speed, y: forward_speed, z: forward_speed });
    }

    if keyboard_input.pressed(KeyCode::Down) {
        ext_force.force = transform.forward().mul(Vec3 { x: backward_speed, y: backward_speed, z: backward_speed });
    }

    let torque: f32 = 32.0;

    if keyboard_input.pressed(KeyCode::Left) {
        ext_force.torque = transform.rotation * Vec3::new(0.0, torque, 0.0);
    }

    if keyboard_input.pressed(KeyCode::Right) {
        ext_force.torque = transform.rotation * Vec3::new(0.0, -torque, 0.0);
    }
}


fn camera_target_car_system(
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
    game.camera_target.look_at = Some(car_transform.translation);
    game.camera_target.up = Some(car_transform.up());
    game.camera_target.position = Some(car_transform.translation + car_transform.forward() * -20.0 + (car_transform.up() * 5.0));
}

fn camera_target_target_system(
    mut transforms: Query<&mut Transform>,
    game: ResMut<Game>,
) {
    let camera_entity = match game.camera { Some(x) => x, _ => { return; } };
    let mut camera_transform = match transforms.get_mut(camera_entity) { Ok(x) => x, _ => { return; } };
    let camera_target_look_at = match game.camera_target.look_at { Some(x) => x, _ => { return; } };
    let camera_target_position = match game.camera_target.position { Some(x) => x, _ => { return; } };
    let camera_target_up = match game.camera_target.up { Some(x) => x, _ => { return; } };

    camera_transform.look_at(camera_target_look_at, camera_target_up);
    camera_transform.translation = camera_target_position;
}

fn setup_dynamic_objects(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    // Create the ground.
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    let my_gltf = asset_server.load("model.glb#Scene0");

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

    let ball_amount_per_dimension = 30;

    for i in 1..ball_amount_per_dimension {
        for j in 1..ball_amount_per_dimension {
            let space = 2.0;
            let offset = space * (ball_amount_per_dimension as f32) / 2.0;
            commands
                .spawn()
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(0.5))
                .insert(Restitution::coefficient(0.7))
                .insert(ColliderMassProperties::Density(0.02))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(i as f32 * space - offset, 4.0, j as f32 * space - offset)));
        }
    }
}

fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        println!("Ball altitude: {}", transform.translation.y);
    }
}
