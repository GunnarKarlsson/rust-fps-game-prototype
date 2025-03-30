use bevy::{
    prelude::*,
    input::{mouse::MouseMotion, keyboard::KeyCode, ButtonInput},
    window::WindowMode,
    math::primitives::{Cuboid, Plane3d},
};

const PLAYER_SPEED: f32 = 5.0;
const MOUSE_SENSITIVITY: f32 = 0.002;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::Windowed,
                title: "Wolfenstein 3D Clone".to_string(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.4, 0.6, 1.0))) // Sky blue background
        .add_systems(Startup, setup)
        .add_systems(Update, (
            player_movement,
            player_look,
            cursor_grab_system,
        ).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load the textures
    let wall_texture = asset_server.load("stone.png");
    let floor_texture = asset_server.load("floor.png");
    
    // Create the wall
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 0.1))),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(wall_texture),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, -5.0),
        ..default()
    });

    // Create the floor
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Plane3d::new(Vec3::Y))),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(floor_texture),
            base_color: Color::WHITE,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, -0.5, 0.0)  // Move down to be at the base of the wall
            .with_scale(Vec3::new(10.0, 1.0, 10.0)),
        ..default()
    });

    // Create a light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Create the camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.6, 0.0).looking_at(Vec3::new(0.0, 1.6, -5.0), Vec3::Y),
            ..default()
        },
        PlayerCamera {
            yaw: 0.0,
            pitch: 0.0,
            position: Vec3::new(0.0, 1.6, 0.0),
        },
    ));
}

#[derive(Component)]
struct PlayerCamera {
    yaw: f32,
    pitch: f32,
    position: Vec3,
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let (mut transform, mut camera) = query.single_mut();
    
    let mut movement = Vec3::ZERO;
    
    if keyboard.pressed(KeyCode::KeyW) {
        movement += Vec3::new(0.0, 0.0, -1.0);
    }
    if keyboard.pressed(KeyCode::KeyS) {
        movement += Vec3::new(0.0, 0.0, 1.0);
    }
    if keyboard.pressed(KeyCode::KeyA) {
        movement += Vec3::new(-1.0, 0.0, 0.0);
    }
    if keyboard.pressed(KeyCode::KeyD) {
        movement += Vec3::new(1.0, 0.0, 0.0);
    }

    if movement != Vec3::ZERO {
        movement = movement.normalize();
        let rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw);
        movement = rotation * movement;
        camera.position += movement * PLAYER_SPEED * time.delta_seconds();
        transform.translation = camera.position;
    }
}

fn player_look(
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera)>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    let (mut transform, mut camera) = camera_query.single_mut();

    for ev in motion_evr.read() {
        camera.yaw -= ev.delta.x * MOUSE_SENSITIVITY;
        camera.pitch -= ev.delta.y * MOUSE_SENSITIVITY;
        camera.pitch = camera.pitch.clamp(-89.0 * std::f32::consts::PI / 180.0, 89.0 * std::f32::consts::PI / 180.0);
    }

    let rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw) * Quat::from_axis_angle(Vec3::X, camera.pitch);
    transform.rotation = rotation;
}

fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
