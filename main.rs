use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

#[derive(Default)]
struct Game {
    player_car: Option<Entity>,
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
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn keyboard_input_system(keyboard_input: Res<Input<KeyCode>>, time: Res<Time>, mut transforms: Query<&mut Transform>, game: ResMut<Game>) {
    if keyboard_input.pressed(KeyCode::W) {
        if let Some(entity) = game.player_car {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.rotate_y(time.delta_seconds());
                transform.scale = Vec3::splat(1.0 + (1.0 / 10.0 * time.delta_seconds().sin()).abs());
            }
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
    game.player_car = Some(commands.spawn_bundle(SceneBundle {
        scene: my_gltf,
        transform: Transform::from_xyz(2.0, 0.0, -5.0),
        ..Default::default()
    }).id());

    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)));
}

fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        println!("Ball altitude: {}", transform.translation.y);
    }
}
