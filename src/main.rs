use bevy::{
    prelude::*,
    input::{mouse::MouseMotion, keyboard::KeyCode, ButtonInput},
    window::WindowMode,
    math::primitives::{Cuboid, Plane3d},
    render::{
        render_resource::{AddressMode},
        texture::{ImageSampler, ImageSamplerDescriptor, ImageAddressMode, ImageFilterMode},
    },
};

const PLAYER_SPEED: f32 = 5.0;
const MOUSE_SENSITIVITY: f32 = 0.002;
const GRID_SIZE: usize = 10;
const TILE_SIZE: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.3; // Player's collision radius

#[derive(Component)]
struct Wall {
    grid_x: usize,
    grid_y: usize,
    side: WallSide,
}

#[derive(PartialEq)]
enum WallSide {
    North,  // Positive Z
    South,  // Negative Z
    East,   // Positive X
    West,   // Negative X
}

// Define the grid layout here
const GRID_LAYOUT: [[bool; GRID_SIZE]; GRID_SIZE] = [
    [false, false, true, true, true, true, true, true, false, false],
    [false, false, false, false, false, false, false, false, false, false],
    [false, false, false, false, false, false, false, true, false, true],
    [true, false, false, false, false, false, false, false, false, true],
    [true, false, false, false, false, false, false, false, false, true],
    [true, false, false, false, true, false, false, false, false, true],
    [true, false, false, false, false, false, false, false, false, true],
    [true, false, true, false, false, false, false, true, false, true],
    [false, false, false, false, false, false, false, false, false, false],
    [false, true, true, true, true, true, true, true, true, false],
];

fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    wall_texture: Handle<Image>,
    grid_x: usize,
    grid_y: usize,
    side: WallSide,
    height: f32, // 0.0 for bottom wall, 1.0 for top wall
) {
    let grid_offset = (GRID_SIZE as f32 * TILE_SIZE) / 2.0;
    let (position, dimensions) = match side {
        WallSide::North => (
            Vec3::new(
                (grid_x as f32 * TILE_SIZE) + (TILE_SIZE / 2.0) - grid_offset,
                height, // Add height offset
                (grid_y as f32 * TILE_SIZE) + TILE_SIZE - grid_offset,
            ),
            Vec3::new(TILE_SIZE, 1.0, 0.1),
        ),
        WallSide::South => (
            Vec3::new(
                (grid_x as f32 * TILE_SIZE) + (TILE_SIZE / 2.0) - grid_offset,
                height, // Add height offset
                grid_y as f32 * TILE_SIZE - grid_offset,
            ),
            Vec3::new(TILE_SIZE, 1.0, 0.1),
        ),
        WallSide::East => (
            Vec3::new(
                (grid_x as f32 * TILE_SIZE) + TILE_SIZE - grid_offset,
                height, // Add height offset
                (grid_y as f32 * TILE_SIZE) + (TILE_SIZE / 2.0) - grid_offset,
            ),
            Vec3::new(0.1, 1.0, TILE_SIZE),
        ),
        WallSide::West => (
            Vec3::new(
                grid_x as f32 * TILE_SIZE - grid_offset,
                height, // Add height offset
                (grid_y as f32 * TILE_SIZE) + (TILE_SIZE / 2.0) - grid_offset,
            ),
            Vec3::new(0.1, 1.0, TILE_SIZE),
        ),
    };

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(dimensions.x, dimensions.y, dimensions.z))),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(wall_texture.clone()),
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        Wall {
            grid_x,
            grid_y,
            side,
        },
    ));
}

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
        .add_systems(Startup, (setup, center_cursor))
        .add_systems(Update, (
            player_movement,
            player_look,
            cursor_grab_system,
            quit_system,
        ).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // Load the textures
    let wall_texture = asset_server.load("stone.png");
    let floor_texture = asset_server.load("floor.png");
    
    // Configure floor texture to repeat
    if let Some(texture) = images.get_mut(&floor_texture) {
        texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: bevy::render::texture::ImageFilterMode::Linear,
            min_filter: bevy::render::texture::ImageFilterMode::Linear,
            mipmap_filter: bevy::render::texture::ImageFilterMode::Linear,
            ..default()
        });
    }

    // Create walls based on the grid layout
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            if GRID_LAYOUT[y][x] {
                // Check each side and spawn walls as needed
                if y == 0 || !GRID_LAYOUT[y-1][x] {
                    // Spawn bottom wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::South, 0.0);
                    // Spawn top wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::South, 1.0);
                }
                if y == GRID_SIZE-1 || !GRID_LAYOUT[y+1][x] {
                    // Spawn bottom wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::North, 0.0);
                    // Spawn top wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::North, 1.0);
                }
                if x == 0 || !GRID_LAYOUT[y][x-1] {
                    // Spawn bottom wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::West, 0.0);
                    // Spawn top wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::West, 1.0);
                }
                if x == GRID_SIZE-1 || !GRID_LAYOUT[y][x+1] {
                    // Spawn bottom wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::East, 0.0);
                    // Spawn top wall
                    spawn_wall(&mut commands, &mut meshes, &mut materials, wall_texture.clone(), x, y, WallSide::East, 1.0);
                }
            }
        }
    }

    // Create the floor as a grid of tiles
    for x in -5..5 {
        for z in -5..5 {
            // Floor
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(Plane3d::new(Vec3::Y))),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(floor_texture.clone()),
                    base_color: Color::WHITE,
                    alpha_mode: AlphaMode::Opaque,
                    double_sided: true,
                    ..default()
                }),
                transform: Transform::from_xyz(x as f32, -0.5, z as f32),
                ..default()
            });

            // Ceiling (2 units above the floor, facing down)
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(Plane3d::new(-Vec3::Y))),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(floor_texture.clone()),
                    base_color: Color::WHITE,
                    alpha_mode: AlphaMode::Opaque,
                    double_sided: true,
                    ..default()
                }),
                transform: Transform::from_xyz(x as f32, 1.5, z as f32),
                ..default()
            });
        }
    }

    // Create a directional light for shadows
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.0, // Reduced from default
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Create interior point lights
    let grid_offset = (GRID_SIZE as f32 * TILE_SIZE) / 2.0;
    
    // First light in the first open space
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(1.0, 0.0, 0.0), // Pure orange light
            intensity: 500000.0, // Increased from 2000.0
            range: 5.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(
            (2.0 * TILE_SIZE) - grid_offset,
            0.5, // Halfway between floor and ceiling
            (2.0 * TILE_SIZE) - grid_offset,
        ),
        ..default()
    });

    // Second light in the second open space
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(0.0, 0.0, 1.0), // Pure blue light
            intensity: 500000.0, // Increased from 2000.0
            range: 5.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(
            (8.0 * TILE_SIZE) - grid_offset,
            0.5, // Halfway between floor and ceiling
            (4.0 * TILE_SIZE) - grid_offset,
        ),
        ..default()
    });

    // Create the camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.5, 2.0).looking_at(Vec3::new(0.0, 1.6, 0.0), Vec3::Y),
            ..default()
        },
        PlayerCamera {
            yaw: 0.0,
            pitch: 0.0,
            position: Vec3::new(0.0, 0.5, 2.0),
        },
    ));
}

#[derive(Component)]
struct PlayerCamera {
    yaw: f32,
    pitch: f32,
    position: Vec3,
}

fn world_to_grid(world_pos: Vec3) -> (usize, usize) {
    let grid_offset = (GRID_SIZE as f32 * TILE_SIZE) / 2.0;
    let x = ((world_pos.x + grid_offset) / TILE_SIZE).floor() as usize;
    let z = ((world_pos.z + grid_offset) / TILE_SIZE).floor() as usize;
    (x, z)
}

fn is_wall_at_position(x: usize, z: usize) -> bool {
    if x >= GRID_SIZE || z >= GRID_SIZE {
        return true; // Treat out of bounds as walls
    }
    GRID_LAYOUT[z][x]
}

fn check_collision(current_pos: Vec3, movement: Vec3) -> bool {
    let grid_offset = (GRID_SIZE as f32 * TILE_SIZE) / 2.0;
    let (current_x, current_z) = world_to_grid(current_pos);
    
    // Check multiple points along the movement vector
    let steps = 4;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let check_pos = current_pos + movement * t;
        
        // Check a few points around the player's radius
        let radius_points = [
            Vec3::new(PLAYER_RADIUS, 0.0, 0.0),
            Vec3::new(-PLAYER_RADIUS, 0.0, 0.0),
            Vec3::new(0.0, 0.0, PLAYER_RADIUS),
            Vec3::new(0.0, 0.0, -PLAYER_RADIUS),
        ];
        
        for offset in radius_points.iter() {
            let check_point = check_pos + *offset;
            let (grid_x, grid_z) = world_to_grid(check_point);
            
            if is_wall_at_position(grid_x, grid_z) {
                println!("Collision detected at grid position: ({}, {})", grid_x, grid_z);
                return true;
            }
        }
    }
    
    false
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PlayerCamera)>,
) {
    let (mut transform, mut camera) = query.single_mut();
    
    let mut movement = Vec3::ZERO;
    
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        movement += Vec3::new(0.0, 0.0, -1.0);
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        movement += Vec3::new(0.0, 0.0, 1.0);
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        movement += Vec3::new(-1.0, 0.0, 0.0);
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        movement += Vec3::new(1.0, 0.0, 0.0);
    }

    if movement != Vec3::ZERO {
        movement = movement.normalize();
        let rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw);
        movement = rotation * movement;
        
        // Calculate the movement for this frame
        let frame_movement = movement * PLAYER_SPEED * time.delta_seconds();
        
        // Check for collisions before applying movement
        if !check_collision(camera.position, frame_movement) {
            camera.position += frame_movement;
            transform.translation = camera.position;
            println!("Player position: {:?}", camera.position);
        }
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

fn center_cursor(mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
}

fn quit_system(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        std::process::exit(0);
    }
}
