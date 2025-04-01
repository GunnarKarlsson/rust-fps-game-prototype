use bevy::{
    input::{keyboard::KeyCode, mouse::MouseMotion, ButtonInput},
    math::primitives::{Cuboid, Plane3d, Sphere},
    prelude::*,
    render::texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    window::WindowMode,
    reflect::TypePath,
};
use bevy_common_assets::json::JsonAssetPlugin;

const PLAYER_SPEED: f32 = 5.0;
const MOUSE_SENSITIVITY: f32 = 0.002;
const GRID_SIZE: usize = 20;
const TILE_SIZE: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.3; // Player's collision radius
const BULLET_SPEED: f32 = 8.0; // Doubled from 0.5 to 1.0 meters per second
const BULLET_LIFETIME: f32 = 10.0; // 10 meters at 1.0 m/s = 10 seconds
const BULLET_SIZE: f32 = 0.1; // Size of the bullet
const BULLET_LIGHT_INTENSITY: f32 = 300000.0;
const BULLET_LIGHT_RANGE: f32 = 3.0;
const ENEMY_SIZE: f32 = 0.3;
const ENEMY_SPEED: f32 = 2.0;
const ENEMY_COLLISION_RADIUS: f32 = 0.5; // Larger than PLAYER_RADIUS (0.3)
const ENEMY_SHOOT_RATE: f32 = 4.0; // Changed from 1.0 to 2.0 seconds between shots
const ENEMY_BULLET_SPEED: f32 = 3.0; // Changed from 8.0 to 6.0 meters per second
const ENEMY_BULLET_SIZE: f32 = 0.1;
const ENEMY_BULLET_LIFETIME: f32 = 5.0;
const ENEMY_BULLET_HIT_RADIUS: f32 = 1.0; // Larger radius for bullet hit detection

// Add these constants for the minimap
const MINIMAP_SIZE: f32 = 150.0; // Size in pixels
const MINIMAP_PADDING: f32 = 20.0; // Padding from screen edges
const MINIMAP_DOT_SIZE: f32 = 6.0; // Size of player/enemy dots

#[derive(serde::Deserialize, Asset, TypePath, Debug)]
struct Level {
    grid_layout: Vec<[bool; 20]>,
}

#[derive(Resource)]
struct LevelHandle(Handle<Level>);

#[derive(Component)]
struct Wall {}

#[derive(PartialEq)]
enum WallSide {
    North, // Positive Z
    South, // Negative Z
    East,  // Positive X
    West,  // Negative X
}

// Define the grid layout here - 20x20 with original pattern in center
const GRID_LAYOUT: [[bool; GRID_SIZE]; GRID_SIZE] = [
    [
        true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, true, true, true, true, true, false, false, true, true, true, true,
        true, true, true, false, false, true,
    ],
    [
        true, false, false, false, false, true, false, false, false, false, true, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, true, false, false, false, false, true, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, true, false, false, false, false, true, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, true, false, false, false, false, true, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, true, true, true, true, true, true, true, false, false,
        false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, true, true, true, true, true, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, true, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, true, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, true, false, false, false, false, true,
    ],
    [
        true, false, false, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, true,
    ],
    [
        true, true, true, true, true, true, true, true, true, true, true, true, true, true, true,
        true, true, true, true, true,
    ],
];

#[derive(Resource)]
struct CurrentLevel {
    number: u32,
}

#[derive(Component)]
struct Bullet {
    velocity: Vec3,
    lifetime: f32,
    light: Entity, // Reference to the light entity
}

#[derive(Component)]
struct Particle {
    velocity: Vec3,
    lifetime: f32,
}

const PARTICLE_COUNT: usize = 12;
const PARTICLE_SIZE: f32 = 0.05;
const PARTICLE_SPEED: f32 = 3.0;
const PARTICLE_LIFETIME: f32 = 0.5;
const PARTICLE_LIGHT_INTENSITY: f32 = 100000.0;
const PARTICLE_LIGHT_RANGE: f32 = 2.0;

#[derive(Component)]
struct Enemy {
    velocity: Vec3,
    last_direction_change: f32,
    shoot_cooldown: f32,
}

#[derive(Component)]
struct EnemyRotation {
    rotation_speed: f32,
}

#[derive(Component)]
struct EnemyBullet {
    velocity: Vec3,
    lifetime: f32,
}

#[derive(Resource)]
struct GameState {
    is_game_over: bool,
    has_won: bool,
    is_level_complete: bool,
    current_level: u32,
    player_health: u32,
    has_started: bool,
}

// Add this component to identify minimap entities
#[derive(Component)]
struct Minimap;

// Add this component for minimap dots
#[derive(Component)]
enum MinimapDot {
    Player,
    Enemy,
}

#[derive(Component)]
struct Gun;

// Add this component for the level display
#[derive(Component)]
struct LevelDisplay;

// Add this component for the health display
#[derive(Component)]
struct HealthDisplay;

#[derive(Component)]
struct HealthPickup {
    health_amount: u32,
}

// Add this component after the other component definitions
#[derive(Component)]
struct StartScreen;

// Add this component after the other component definitions
#[derive(Component)]
struct HealthPickupRotation {
    rotation_speed: f32,
}

#[derive(Component)]
struct DamageFlash {
    lifetime: f32,
}

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
            mesh: meshes.add(Mesh::from(Cuboid::new(
                dimensions.x,
                dimensions.y,
                dimensions.z,
            ))),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(wall_texture.clone()),
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        Wall {},
    ));
}

fn spawn_enemy(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
) {
    error!("DEBUG: Starting enemy spawn at position: {:?}", position);
    
    // Create parent entity with Enemy component and transform
    commands.spawn((
        Enemy {
            velocity: Vec3::new(0.0, 0.0, -1.0) * ENEMY_SPEED,
            last_direction_change: 0.0,
            shoot_cooldown: 0.0,
        },
        EnemyRotation {
            rotation_speed: 0.5, // Rotate 0.5 radians per second
        },
        SpatialBundle {
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        },
    ))
    .with_children(|parent| {
        error!("DEBUG: Attempting to spawn skull model as child");
        parent.spawn(SceneBundle {
            scene: asset_server.load("models/skull.glb#Scene0"),
            transform: Transform::from_scale(Vec3::splat(0.5)),
            ..default()
        });

        // Add blue light below the skull
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(0.0, 0.0, 1.0), // Blue light
                intensity: 10000.0,
                range: 2.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, -0.4, 0.0), // 0.4 meters below the skull (0.5 - 0.1)
            ..default()
        });

        error!("DEBUG: Finished spawning skull model and light");
    });
}

fn is_valid_spawn_position(x: usize, z: usize) -> bool {
    // Check if position is within grid bounds
    if x >= GRID_SIZE || z >= GRID_SIZE {
        return false;
    }
    
    // Check if position is not a wall
    if GRID_LAYOUT[z][x] {
        return false;
    }
    
    // Check surrounding tiles to ensure there's enough space
    let directions = [
        (0, 1), (1, 1), (1, 0), (1, -1),
        (0, -1), (-1, -1), (-1, 0), (-1, 1)
    ];
    
    for (dx, dz) in directions.iter() {
        let check_x = x as i32 + dx;
        let check_z = z as i32 + dz;
        
        // Skip if out of bounds
        if check_x < 0 || check_x >= GRID_SIZE as i32 || check_z < 0 || check_z >= GRID_SIZE as i32 {
            continue;
        }
        
        // If any adjacent tile is a wall, this might be too cramped
        if GRID_LAYOUT[check_z as usize][check_x as usize] {
            return false;
        }
    }
    
    true
}

fn reset_game(
    commands: &mut Commands,
    mut game_state: ResMut<GameState>,
    mut player_query: Query<(&mut Transform, &mut PlayerCamera)>,
    enemy_query: Query<Entity, With<Enemy>>,
    bullet_query: Query<Entity, Or<(With<Bullet>, With<EnemyBullet>)>>,
    pickup_query: Query<Entity, With<HealthPickup>>,
    asset_server: Res<AssetServer>,
) {
    // Reset game state
    game_state.is_game_over = false;
    game_state.has_won = false;
    game_state.is_level_complete = false;
    game_state.player_health = 100;
    game_state.has_started = true;

    // Reset player position and rotation
    if let Ok((mut transform, mut camera)) = player_query.get_single_mut() {
        camera.position = Vec3::new(0.0, 0.5, 2.0);
        camera.yaw = 0.0;
        camera.pitch = 0.0;
        transform.translation = camera.position;
        transform.rotation = Quat::IDENTITY;
    }

    // Remove all existing enemies and their children
    for entity in enemy_query.iter() {
        if let Some(mut enemy) = commands.get_entity(entity) {
            enemy.despawn_recursive();
        }
    }

    // Remove all bullets and their children
    for entity in bullet_query.iter() {
        if let Some(mut bullet) = commands.get_entity(entity) {
            bullet.despawn_recursive();
        }
    }

    // Remove all existing health pickups
    for entity in pickup_query.iter() {
        if let Some(mut pickup) = commands.get_entity(entity) {
            pickup.despawn_recursive();
        }
    }

    // Calculate grid offset for spawn positions
    let grid_offset = (GRID_SIZE as f32 * TILE_SIZE) / 2.0;

    // Spawn enemies based on current level
    let enemy_count = game_state.current_level as usize;
    let mut enemies_spawned = 0;
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 100; // Prevent infinite loop

    while enemies_spawned < enemy_count && attempts < MAX_ATTEMPTS {
        // Generate random position
        let x = rand::random::<usize>() % GRID_SIZE;
        let z = rand::random::<usize>() % GRID_SIZE;

        if is_valid_spawn_position(x, z) {
            // Convert grid position to world position
            let world_pos = Vec3::new(
                (x as f32 * TILE_SIZE) - grid_offset,
                0.5, // Halfway between floor and ceiling
                (z as f32 * TILE_SIZE) - grid_offset,
            );

            // Spawn enemy at valid position
            spawn_enemy(
                commands,
                &asset_server,
                world_pos,
            );
            enemies_spawned += 1;
        }
        attempts += 1;
    }

    // If we couldn't spawn all enemies, log a warning
    if enemies_spawned < enemy_count {
        error!("Could only spawn {} out of {} enemies due to space constraints", 
            enemies_spawned, enemy_count);
    }

    // Spawn health pickups
    let mut pickups_spawned = 0;
    attempts = 0;

    while pickups_spawned < 2 && attempts < MAX_ATTEMPTS {
        // Generate random position
        let x = rand::random::<usize>() % GRID_SIZE;
        let z = rand::random::<usize>() % GRID_SIZE;

        if is_valid_spawn_position(x, z) {
            // Convert grid position to world position
            let world_pos = Vec3::new(
                (x as f32 * TILE_SIZE) - grid_offset,
                0.5, // Halfway between floor and ceiling
                (z as f32 * TILE_SIZE) - grid_offset,
            );

            // Spawn either bottle or soda bottle based on which one we need
            let model_path = if pickups_spawned == 0 {
                "models/bottle.glb#Scene0"
            } else {
                "models/soda-bottle.glb#Scene0"
            };

            commands.spawn((
                SceneBundle {
                    scene: asset_server.load(model_path),
                    transform: Transform::from_translation(world_pos)
                        .with_scale(Vec3::splat(1.0))
                        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_4)), // 45-degree tilt
                    ..default()
                },
                HealthPickup {
                    health_amount: 20,
                },
                HealthPickupRotation {
                    rotation_speed: 1.0, // Rotate 1 radian per second
                },
            )).with_children(|parent| {
                // Add green light above the bottle
                parent.spawn(PointLightBundle {
                    point_light: PointLight {
                        color: Color::rgb(0.0, 1.0, 0.0), // Green light
                        intensity: 50000.0,
                        range: 3.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 1.0, 0.0), // 1 unit above the bottle
                    ..default()
                });
            });

            pickups_spawned += 1;
        }
        attempts += 1;
    }

    // If we couldn't spawn all health pickups, log a warning
    if pickups_spawned < 2 {
        error!("Could only spawn {} out of 2 health pickups due to space constraints", 
            pickups_spawned);
    }
}

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        JsonAssetPlugin::<Level>::new(&["level.json"]),
    ))
        .insert_resource(ClearColor(Color::rgb(0.4, 0.6, 1.0)))
        .insert_resource(GameState { 
            is_game_over: false,
            has_won: false,
            is_level_complete: false,
            current_level: 1,
            player_health: 100,
            has_started: false,
        })
        .insert_resource(CurrentLevel { number: 1 })
        .add_systems(Startup, (setup, center_cursor, spawn_minimap, spawn_health_pickups, spawn_start_screen))
        .add_systems(
            Update,
            (
                start_screen_system,
                player_movement.run_if(|state: Res<GameState>| state.has_started && !state.is_game_over && !state.is_level_complete),
                player_look.run_if(|state: Res<GameState>| state.has_started && !state.is_game_over && !state.is_level_complete),
                cursor_grab_system.run_if(|state: Res<GameState>| state.has_started && !state.is_game_over && !state.is_level_complete),
                shoot_bullet.run_if(|state: Res<GameState>| state.has_started && !state.is_game_over && !state.is_level_complete),
                update_bullets.run_if(|state: Res<GameState>| state.has_started),
                update_particles.run_if(|state: Res<GameState>| state.has_started),
                update_enemies.run_if(|state: Res<GameState>| state.has_started),
                enemy_shooting.run_if(|state: Res<GameState>| state.has_started),
                update_enemy_bullets.run_if(|state: Res<GameState>| state.has_started),
                update_minimap.run_if(|state: Res<GameState>| state.has_started),
                game_over_ui.run_if(|state: Res<GameState>| state.has_started),
                restart_system.run_if(|state: Res<GameState>| state.has_started),
                quit_system.run_if(|state: Res<GameState>| state.has_started),
                update_health_pickups.run_if(|state: Res<GameState>| state.has_started),
                update_damage_flash.run_if(|state: Res<GameState>| state.has_started),
            )
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {

    // Load the level json
    let level = LevelHandle(asset_server.load("1.level.json"));
    commands.insert_resource(level);

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
                if y == 0 || !GRID_LAYOUT[y - 1][x] {
                    // Spawn bottom wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::South,
                        0.0,
                    );
                    // Spawn top wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::South,
                        1.0,
                    );
                }
                if y == GRID_SIZE - 1 || !GRID_LAYOUT[y + 1][x] {
                    // Spawn bottom wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::North,
                        0.0,
                    );
                    // Spawn top wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::North,
                        1.0,
                    );
                }
                if x == 0 || !GRID_LAYOUT[y][x - 1] {
                    // Spawn bottom wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::West,
                        0.0,
                    );
                    // Spawn top wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::West,
                        1.0,
                    );
                }
                if x == GRID_SIZE - 1 || !GRID_LAYOUT[y][x + 1] {
                    // Spawn bottom wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::East,
                        0.0,
                    );
                    // Spawn top wall
                    spawn_wall(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        wall_texture.clone(),
                        x,
                        y,
                        WallSide::East,
                        1.0,
                    );
                }
            }
        }
    }

    // Create the floor as a grid of tiles - extended to match new grid size
    for x in -10..10 {
        for z in -10..10 {
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
            illuminance: 200.0, // Reduced from 1000.0 to make it darker
            color: Color::rgb(0.5, 0.5, 1.0), // Slightly blueish tint
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
            intensity: 200000.0,              // Increased from 2000.0
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
            intensity: 200000.0,              // Increased from 2000.0
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

    // Create the camera with gun
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.5, 2.0)
                .looking_at(Vec3::new(0.0, 1.6, 0.0), Vec3::Y),
            ..default()
        },
        PlayerCamera {
            yaw: 0.0,
            pitch: 0.0,
            position: Vec3::new(0.0, 0.5, 2.0),
        },
    ))
    .with_children(|parent| {
        // Spawn the gun model
        parent.spawn((
            SceneBundle {
                scene: asset_server.load("models/blaster-g.glb#Scene0"),
                transform: Transform::from_xyz(0.3, -0.2, -0.5) // Position in front and slightly to the right
                    .with_scale(Vec3::splat(0.5)), // Adjust scale as needed
                ..default()
            },
            Gun,
        ));
    });

    // Spawn an enemy in an open space
    spawn_enemy(
        &mut commands,
        &asset_server,
        Vec3::new(
            (2.0 * TILE_SIZE) - grid_offset,
            0.5, // 0.5 meters above the floor
            (2.0 * TILE_SIZE) - grid_offset,
        ),
    );
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
                //println!(
                //    "Collision detected at grid position: ({}, {})",
                //    grid_x, grid_z
                //);
                return true;
            }
        }
    }

    false
}

fn check_enemy_collision(current_pos: Vec3, movement: Vec3) -> bool {
    // Check multiple points along the movement vector
    let steps = 4;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let check_pos = current_pos + movement * t;

        // Check more points around the enemy's radius for better collision detection
        let radius_points = [
            Vec3::new(ENEMY_COLLISION_RADIUS, 0.0, 0.0),
            Vec3::new(-ENEMY_COLLISION_RADIUS, 0.0, 0.0),
            Vec3::new(0.0, 0.0, ENEMY_COLLISION_RADIUS),
            Vec3::new(0.0, 0.0, -ENEMY_COLLISION_RADIUS),
            // Add diagonal points for better coverage
            Vec3::new(ENEMY_COLLISION_RADIUS * 0.7, 0.0, ENEMY_COLLISION_RADIUS * 0.7),
            Vec3::new(-ENEMY_COLLISION_RADIUS * 0.7, 0.0, ENEMY_COLLISION_RADIUS * 0.7),
            Vec3::new(ENEMY_COLLISION_RADIUS * 0.7, 0.0, -ENEMY_COLLISION_RADIUS * 0.7),
            Vec3::new(-ENEMY_COLLISION_RADIUS * 0.7, 0.0, -ENEMY_COLLISION_RADIUS * 0.7),
        ];

        for offset in radius_points.iter() {
            let check_point = check_pos + *offset;
            let (grid_x, grid_z) = world_to_grid(check_point);

            if is_wall_at_position(grid_x, grid_z) {
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
            //println!("Player position: {:?}", camera.position);
        }
    }
}

fn player_look(
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera, &Children)>,
    mut motion_evr: EventReader<MouseMotion>,
    mut gun_query: Query<&mut Transform, (With<Gun>, Without<PlayerCamera>)>,
) {
    let (mut transform, mut camera, children) = camera_query.single_mut();

    for ev in motion_evr.read() {
        camera.yaw -= ev.delta.x * MOUSE_SENSITIVITY;
        camera.pitch -= ev.delta.y * MOUSE_SENSITIVITY;
        camera.pitch = camera.pitch.clamp(
            -89.0 * std::f32::consts::PI / 180.0,
            89.0 * std::f32::consts::PI / 180.0,
        );
    }

    let rotation =
        Quat::from_axis_angle(Vec3::Y, camera.yaw) * Quat::from_axis_angle(Vec3::X, camera.pitch);
    transform.rotation = rotation;

    // Update gun position based on camera rotation
    for child in children.iter() {
        if let Ok(mut gun_transform) = gun_query.get_mut(*child) {
            // Keep the gun's local position relative to the camera
            gun_transform.rotation = Quat::IDENTITY;
        }
    }
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

fn shoot_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Transform, &PlayerCamera)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let (_transform, camera) = camera_query.single();

        // Calculate bullet direction based on camera rotation
        let rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw)
            * Quat::from_axis_angle(Vec3::X, camera.pitch);
        let bullet_direction = rotation * -Vec3::Z; // Negative Z is forward in our coordinate system

        // Set bullet spawn position to the tip of the gun
        let gun_offset = Vec3::new(0.3, -0.2, -0.5); // Gun position relative to camera
        let bullet_position = camera.position + rotation * gun_offset;
        error!("DEBUG: Attempting to spawn bullet at position: {:?}", bullet_position);

        // Create parent entity with Bullet component and transform
        let bullet_entity = commands
            .spawn((
                Bullet {
                    velocity: bullet_direction * BULLET_SPEED,
                    lifetime: BULLET_LIFETIME,
                    light: Entity::PLACEHOLDER,
                },
                SpatialBundle {
                    transform: Transform::from_translation(bullet_position)
                        .with_rotation(rotation),
                    ..default()
                },
            ))
            .with_children(|parent| {
                error!("DEBUG: Attempting to spawn bullet model");
                parent.spawn(SceneBundle {
                    scene: asset_server.load("models/bullet-foam.glb#Scene0"),
                    transform: Transform::from_scale(Vec3::splat(2.0))
                        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_4 * 2.0)), // 45-degree rotation around X-axis
                    ..default()
                });
                error!("DEBUG: Finished spawning bullet model");
            })
            .id();

        error!("DEBUG: Spawned bullet entity with ID: {:?}", bullet_entity);

        // Spawn light for the bullet
        let light_entity = commands
            .spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    intensity: BULLET_LIGHT_INTENSITY,
                    range: BULLET_LIGHT_RANGE,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::from_translation(bullet_position),
                ..default()
            })
            .id();

        error!("DEBUG: Spawned bullet light with ID: {:?}", light_entity);

        // Update bullet with light entity reference
        if let Some(mut bullet) = commands.get_entity(bullet_entity) {
            bullet.insert(Bullet {
                velocity: bullet_direction * BULLET_SPEED,
                lifetime: BULLET_LIFETIME,
                light: light_entity,
            });
            error!("DEBUG: Updated bullet with light reference");
        }
    }
}

fn spawn_particle_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
) {
    for i in 0..PARTICLE_COUNT {
        // Calculate random direction in a sphere
        let angle = (i as f32 / PARTICLE_COUNT as f32) * 2.0 * std::f32::consts::PI;
        let pitch = (rand::random::<f32>() - 0.5) * std::f32::consts::PI;
        let direction = Vec3::new(
            pitch.cos() * angle.cos(),
            pitch.sin(),
            pitch.cos() * angle.sin(),
        );

        // Spawn particle
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid::new(
                    PARTICLE_SIZE,
                    PARTICLE_SIZE,
                    PARTICLE_SIZE,
                ))),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color * 50.0,
                    ..default()
                }),
                transform: Transform::from_translation(position),
                ..default()
            },
            Particle {
                velocity: direction * PARTICLE_SPEED,
                lifetime: PARTICLE_LIFETIME,
            },
        ));
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_query: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    for (entity, mut transform, mut particle) in particle_query.iter_mut() {
        // Update position
        transform.translation += particle.velocity * time.delta_seconds();

        // Update lifetime
        particle.lifetime -= time.delta_seconds();

        // Remove particle if lifetime is up
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullet_query: Query<(Entity, &mut Transform, &mut Bullet)>,
    mut light_query: Query<&mut Transform, (Without<Bullet>, Without<Enemy>)>,
    enemy_query: Query<(Entity, &Transform, &Children), (With<Enemy>, Without<Bullet>)>,
    mut game_state: ResMut<GameState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut transform, mut bullet) in bullet_query.iter_mut() {
        // Update position
        let new_position = transform.translation + bullet.velocity * time.delta_seconds();
        
        // Check for enemy collision using the larger hit radius
        for (enemy_entity, enemy_transform, children) in enemy_query.iter() {
            let distance = enemy_transform.translation.distance(transform.translation);
            if distance < (ENEMY_BULLET_HIT_RADIUS + BULLET_SIZE) {
                // Check if this is the last enemy before removing it
                if enemy_query.iter().count() == 1 {
                    game_state.is_level_complete = true;
                }

                // Spawn particle explosion at enemy position
                spawn_particle_explosion(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    enemy_transform.translation,
                    Color::rgb(1.0, 0.0, 0.0), // Red particles for enemy destruction
                );
                
                // Remove all children first
                for &child in children.iter() {
                    commands.entity(child).despawn_recursive();
                }
                
                // Then remove the enemy entity
                commands.entity(enemy_entity).despawn();
                
                // Remove bullet, its light, and all children
                commands.entity(bullet.light).despawn();
                commands.entity(entity).despawn_recursive();
                break; // Break out of the enemy loop since we've hit one
            }
        }
        
        // Check for wall collision
        let (grid_x, grid_z) = world_to_grid(new_position);
        if is_wall_at_position(grid_x, grid_z) {
            // Spawn particle explosion at collision point
            spawn_particle_explosion(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                Color::rgb(1.0, 0.0, 0.0), // Red particles for wall collision
            );
            
            // Remove bullet, its light, and all children
            commands.entity(bullet.light).despawn();
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Check for floor and ceiling collisions
        if new_position.y <= -0.5 || new_position.y >= 1.5 {
            // Spawn particle explosion at collision point
            spawn_particle_explosion(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                Color::rgb(1.0, 0.0, 0.0), // Red particles for floor/ceiling collision
            );
            
            // Remove bullet, its light, and all children
            commands.entity(bullet.light).despawn();
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Update position if no collision
        transform.translation = new_position;

        // Update light position
        if let Ok(mut light_transform) = light_query.get_mut(bullet.light) {
            light_transform.translation = transform.translation;
        }

        // Update lifetime
        bullet.lifetime -= time.delta_seconds();

        // Remove bullet, its light, and all children if lifetime is up
        if bullet.lifetime <= 0.0 {
            commands.entity(bullet.light).despawn();
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_enemies(
    mut enemy_query: Query<(&mut Transform, &mut Enemy, &EnemyRotation)>,
    time: Res<Time>,
) {
    for (mut transform, mut enemy, rotation) in enemy_query.iter_mut() {
        // Calculate new position
        let movement = enemy.velocity * time.delta_seconds();
        
        // Check for wall collision using the new collision function
        if check_enemy_collision(transform.translation, movement) {
            // Choose random new direction (left or right relative to current direction)
            let current_direction = enemy.velocity.normalize();
            let random_turn = if rand::random::<bool>() {
                // Turn left
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            } else {
                // Turn right
                Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)
            };
            
            let new_direction = random_turn * current_direction;
            enemy.velocity = new_direction * ENEMY_SPEED;
            enemy.last_direction_change = time.elapsed_seconds();
        } else {
            // Update position if no collision
            transform.translation += movement;
        }

        // Update rotation
        transform.rotate_y(rotation.rotation_speed * time.delta_seconds());
    }
}

fn has_line_of_sight(enemy_pos: Vec3, player_pos: Vec3) -> bool {
    let direction = player_pos - enemy_pos;
    let distance = direction.length();
    let ray_steps = (distance / 0.5).ceil() as i32; // Check every 0.5 units
    
    for i in 0..ray_steps {
        let t = i as f32 / ray_steps as f32;
        let check_pos = enemy_pos + direction * t;
        let (grid_x, grid_z) = world_to_grid(check_pos);
        
        if is_wall_at_position(grid_x, grid_z) {
            return false;
        }
    }
    true
}

fn enemy_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    player_query: Query<&Transform, With<PlayerCamera>>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if game_state.is_game_over {
        return;
    }

    let player_transform = player_query.single();
    
    for (enemy_transform, mut enemy) in enemy_query.iter_mut() {
        enemy.shoot_cooldown -= time.delta_seconds();
        
        if enemy.shoot_cooldown <= 0.0 && has_line_of_sight(enemy_transform.translation, player_transform.translation) {
            // Reset cooldown
            enemy.shoot_cooldown = ENEMY_SHOOT_RATE;
            
            // Calculate direction to player
            let direction = (player_transform.translation - enemy_transform.translation).normalize();
            
            // Calculate rotation to point in direction of travel
            let rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            
            // Spawn enemy bullet using the model
            commands.spawn((
                SceneBundle {
                    scene: asset_server.load("models/enemy-bullet.glb#Scene0"),
                    transform: Transform::from_translation(enemy_transform.translation)
                        .with_scale(Vec3::splat(0.5))
                        .with_rotation(rotation),
                    ..default()
                },
                EnemyBullet {
                    velocity: direction * ENEMY_BULLET_SPEED,
                    lifetime: ENEMY_BULLET_LIFETIME,
                },
            )).with_children(|parent| {
                // Add yellow light at the center of the bullet with reduced intensity
                parent.spawn(PointLightBundle {
                    point_light: PointLight {
                        color: Color::rgb(1.0, 1.0, 0.0), // Yellow light
                        intensity: 5000.0, // Reduced from 10000.0 to make it fainter
                        range: 3.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 0.0), // Center of the bullet
                    ..default()
                });
            });
        }
    }
}

fn update_enemy_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullet_query: Query<(Entity, &mut Transform, &mut EnemyBullet, &Children)>,
    player_query: Query<&Transform, (With<PlayerCamera>, Without<EnemyBullet>)>,
    mut game_state: ResMut<GameState>,
    mut flash_query: Query<Entity, With<DamageFlash>>,
) {
    let player_transform = player_query.single();
    
    for (entity, mut transform, mut bullet, children) in bullet_query.iter_mut() {
        // Update position
        let new_position = transform.translation + bullet.velocity * time.delta_seconds();
        
        // Check for player collision
        let distance = player_transform.translation.distance(transform.translation);
        if distance < (PLAYER_RADIUS + ENEMY_BULLET_SIZE) {
            // Reduce health by 20
            if game_state.player_health > 20 {
                game_state.player_health -= 20;
            } else {
                game_state.player_health = 0;
                game_state.is_game_over = true;
            }

            // Remove any existing flash
            for flash_entity in flash_query.iter() {
                commands.entity(flash_entity).despawn();
            }

            // Spawn new flash effect
            commands.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(1.0, 0.0, 0.0, 0.3)), // Semi-transparent red
                    ..default()
                },
                DamageFlash {
                    lifetime: 0.1, // Flash lasts 0.2 seconds
                },
            ));

            // Remove the bullet and all its children (model and light)
            commands.entity(entity).despawn_recursive();
            continue;
        }
        
        // Check for wall collision
        let (grid_x, grid_z) = world_to_grid(new_position);
        if is_wall_at_position(grid_x, grid_z) {
            // Remove the bullet and all its children (model and light)
            commands.entity(entity).despawn_recursive();
            continue;
        }
        
        // Update position if no collision
        transform.translation = new_position;
        
        // Update lifetime
        bullet.lifetime -= time.delta_seconds();
        if bullet.lifetime <= 0.0 {
            // Remove the bullet and all its children (model and light)
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn game_over_ui(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut text_query: Query<(Entity, &mut Text, Option<&LevelDisplay>, Option<&HealthDisplay>)>,
) {
    // Update or spawn level display
    let mut has_level_display = false;
    let mut has_health_display = false;
    
    for (entity, mut text, level_display, health_display) in text_query.iter_mut() {
        if level_display.is_some() {
            has_level_display = true;
            text.sections[0].value = format!("Level {}", game_state.current_level);
        }
        if health_display.is_some() {
            has_health_display = true;
            text.sections[0].value = format!("Health: {}%", game_state.player_health);
        }
    }

    // Spawn level display if it doesn't exist
    if !has_level_display {
        commands.spawn((
            TextBundle::from_section(
                format!("Level {}", game_state.current_level),
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                ..default()
            }),
            LevelDisplay,
        ));
    }

    // Spawn health display if it doesn't exist
    if !has_health_display {
        commands.spawn((
            TextBundle::from_section(
                format!("Health: {}%", game_state.player_health),
                TextStyle {
                    font_size: 30.0,
                    color: Color::WHITE,
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(200.0),
                ..default()
            }),
            HealthDisplay,
        ));
    }

    // Check if we need to spawn game over message
    let has_game_over = text_query.iter().any(|(_, _, level, health)| level.is_none() && health.is_none());
    if (game_state.is_game_over || game_state.is_level_complete) && !has_game_over {
        let message = if game_state.is_game_over {
            "Game Over\nPress P to Play Again\nPress Q to Exit"
        } else {
            &format!("You finished level {}!\nPress S to start level {}\nPress Q to quit", 
                game_state.current_level, 
                game_state.current_level + 1)
        };

        commands.spawn((
            TextBundle::from_section(
                message,
                TextStyle {
                    font_size: 50.0,
                    color: if game_state.is_game_over { Color::RED } else { Color::GREEN },
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            }),
        ));
    }
}

fn restart_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    player_query: Query<(&mut Transform, &mut PlayerCamera)>,
    enemy_query: Query<Entity, With<Enemy>>,
    bullet_query: Query<Entity, Or<(With<Bullet>, With<EnemyBullet>)>>,
    pickup_query: Query<Entity, With<HealthPickup>>,
    game_over_query: Query<Entity, (With<Text>, Without<LevelDisplay>, Without<HealthDisplay>)>,
    asset_server: Res<AssetServer>,
) {
    if (game_state.is_game_over && keyboard.just_pressed(KeyCode::KeyP)) ||
       (game_state.is_level_complete && keyboard.just_pressed(KeyCode::KeyS)) {
        // Remove only game over/level complete text, keep level display
        for entity in game_over_query.iter() {
            commands.entity(entity).despawn();
        }

        if game_state.is_level_complete {
            game_state.current_level += 1;
        } else {
            game_state.current_level = 1;
        }

        // Reset the game
        reset_game(
            &mut commands,
            game_state,
            player_query,
            enemy_query,
            bullet_query,
            pickup_query,
            asset_server,
        );
    }
}

// Add this function to spawn the minimap
fn spawn_minimap(
    mut commands: Commands,
) {
    // Spawn minimap background with lower z-index
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(MINIMAP_PADDING),
                bottom: Val::Px(MINIMAP_PADDING),
                width: Val::Px(MINIMAP_SIZE),
                height: Val::Px(MINIMAP_SIZE),
                ..default()
            },
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5)),
            ..default()
        },
        Minimap,
    ));

    // Spawn grid walls
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            if GRID_LAYOUT[y][x] {
                let cell_size = MINIMAP_SIZE / GRID_SIZE as f32;
                commands.spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(MINIMAP_PADDING + x as f32 * cell_size),
                            bottom: Val::Px(MINIMAP_PADDING + MINIMAP_SIZE - (y as f32 + 1.0) * cell_size),
                            width: Val::Px(cell_size),
                            height: Val::Px(cell_size),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::rgba(0.5, 0.5, 0.5, 0.8)),
                        ..default()
                    },
                    Minimap,
                ));
            }
        }
    }

    // Spawn player dot
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(MINIMAP_DOT_SIZE),
                height: Val::Px(MINIMAP_DOT_SIZE),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(0.0, 1.0, 0.0)),
            ..default()
        },
        MinimapDot::Player,
        Minimap,
    ));

    // Spawn enemy dot
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(MINIMAP_DOT_SIZE),
                height: Val::Px(MINIMAP_DOT_SIZE),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(1.0, 0.0, 0.0)),
            ..default()
        },
        MinimapDot::Enemy,
        Minimap,
    ));
}

// Update the minimap system to handle multiple enemies
fn update_minimap(
    mut commands: Commands,
    mut dot_query: Query<(Entity, &mut Style, &MinimapDot)>,
    player_query: Query<&Transform, (With<PlayerCamera>, Without<Enemy>)>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    let grid_offset = (GRID_SIZE as f32 * TILE_SIZE) / 2.0;
    let cell_size = MINIMAP_SIZE / GRID_SIZE as f32;

    // Keep track of enemy dots we've updated
    let mut enemy_dots = 0;

    // Get player position safely
    let player_pos = if let Ok(player_transform) = player_query.get_single() {
        player_transform.translation
    } else {
        return; // Exit if player doesn't exist
    };

    for (entity, mut style, dot_type) in dot_query.iter_mut() {
        match dot_type {
            MinimapDot::Player => {
                let minimap_x = (player_pos.x + grid_offset) / TILE_SIZE * cell_size;
                let minimap_y = (player_pos.z + grid_offset) / TILE_SIZE * cell_size;
                
                style.left = Val::Px(MINIMAP_PADDING + minimap_x - MINIMAP_DOT_SIZE/2.0);
                style.bottom = Val::Px(MINIMAP_PADDING + MINIMAP_SIZE - minimap_y - MINIMAP_DOT_SIZE/2.0);
            }
            MinimapDot::Enemy => {
                // If we have an enemy transform for this dot, update it
                if let Some(enemy_transform) = enemy_query.iter().nth(enemy_dots) {
                    let pos = enemy_transform.translation;
                    let minimap_x = (pos.x + grid_offset) / TILE_SIZE * cell_size;
                    let minimap_y = (pos.z + grid_offset) / TILE_SIZE * cell_size;
                    
                    style.left = Val::Px(MINIMAP_PADDING + minimap_x - MINIMAP_DOT_SIZE/2.0);
                    style.bottom = Val::Px(MINIMAP_PADDING + MINIMAP_SIZE - minimap_y - MINIMAP_DOT_SIZE/2.0);
                    enemy_dots += 1;
                } else {
                    // If we don't have an enemy for this dot, remove it
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    // Spawn new enemy dots if we need more
    for enemy_transform in enemy_query.iter().skip(enemy_dots) {
        let pos = enemy_transform.translation;
        let minimap_x = (pos.x + grid_offset) / TILE_SIZE * cell_size;
        let minimap_y = (pos.z + grid_offset) / TILE_SIZE * cell_size;

        commands.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(MINIMAP_PADDING + minimap_x - MINIMAP_DOT_SIZE/2.0),
                    bottom: Val::Px(MINIMAP_PADDING + MINIMAP_SIZE - minimap_y - MINIMAP_DOT_SIZE/2.0),
                    width: Val::Px(MINIMAP_DOT_SIZE),
                    height: Val::Px(MINIMAP_DOT_SIZE),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgb(1.0, 0.0, 0.0)),
                ..default()
            },
            MinimapDot::Enemy,
            Minimap,
        ));
    }
}

fn spawn_health_pickups(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let grid_offset = (GRID_SIZE as f32 * TILE_SIZE) / 2.0;
    
    // Spawn first bottle (bottle.glb)
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/bottle.glb#Scene0"),
            transform: Transform::from_xyz(
                (5.0 * TILE_SIZE) - grid_offset,
                0.0, // Halfway between floor and ceiling
                (10.0 * TILE_SIZE) - grid_offset,
            )
            .with_scale(Vec3::splat(1.0))
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_4)), // 45-degree tilt
            ..default()
        },
        HealthPickup {
            health_amount: 20,
        },
        HealthPickupRotation {
            rotation_speed: 1.0, // Rotate 1 radian per second
        },
    )).with_children(|parent| {
        // Add green light above the bottle
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(0.0, 1.0, 0.0), // Green light
                intensity: 50000.0,
                range: 3.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 1.0, 0.0), // 1 unit above the bottle
            ..default()
        });
    });

    // Spawn second bottle (soda-bottle.glb)
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/soda-bottle.glb#Scene0"),
            transform: Transform::from_xyz(
                (5.0 * TILE_SIZE) - grid_offset,
                0.0,
                (12.0 * TILE_SIZE) - grid_offset,
            )
            .with_scale(Vec3::splat(1.0))
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_4)), // 45-degree tilt
            ..default()
        },
        HealthPickup {
            health_amount: 20,
        },
        HealthPickupRotation {
            rotation_speed: 1.0, // Rotate 1 radian per second
        },
    )).with_children(|parent| {
        // Add green light above the soda bottle
        parent.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(0.0, 1.0, 0.0), // Green light
                intensity: 50000.0,
                range: 3.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 1.0, 0.0), // 1 unit above the bottle
            ..default()
        });
    });
}

fn update_health_pickups(
    mut commands: Commands,
    time: Res<Time>,
    mut pickup_query: Query<(Entity, &mut Transform, &HealthPickup, &HealthPickupRotation)>,
    player_query: Query<&Transform, (With<PlayerCamera>, Without<HealthPickup>)>,
    mut game_state: ResMut<GameState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_transform = player_query.single();
    
    for (entity, mut transform, pickup, rotation) in pickup_query.iter_mut() {
        // Update rotation
        transform.rotate_y(rotation.rotation_speed * time.delta_seconds());
        
        let distance = player_transform.translation.distance(transform.translation);
        
        if distance < (PLAYER_RADIUS + 0.5) { // 0.5 is the pickup radius
            // Spawn particle effect with green color
            spawn_particle_explosion(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                Color::rgb(0.0, 1.0, 0.0), // Green particles for health pickup
            );
            
            // Heal the player
            game_state.player_health = (game_state.player_health + pickup.health_amount).min(100);
            
            // Remove the pickup
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Add this new system after the other system definitions
fn start_screen_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    start_screen_query: Query<Entity, With<StartScreen>>,
    level: Res<LevelHandle>,
    levels: ResMut<Assets<Level>>,
) {
    if let Some(level) = levels.get(level.0.id()) {
        println!("level from json: {:?}", level.grid_layout);
        println!("level from constant: {:?}", GRID_LAYOUT);
    }
    if !game_state.has_started && keyboard.just_pressed(KeyCode::KeyS) {
        // Remove all start screen entities (background, title, instructions, and start prompt)
        for entity in start_screen_query.iter() {
            commands.entity(entity).despawn_recursive(); // Use despawn_recursive to remove all children
        }
        game_state.has_started = true;
    }
}

// Add this new function to spawn the start screen
fn spawn_start_screen(mut commands: Commands) {
    // Create a parent node that will contain all start screen elements
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            background_color: BackgroundColor(Color::rgb(0.1, 0.1, 0.1)), // Dark gray background
            ..default()
        },
        StartScreen,
    )).with_children(|parent| {
        // Spawn title text
        parent.spawn(TextBundle::from_section(
            "Infinite Ammo",
            TextStyle {
                font_size: 80.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Auto,
            right: Val::Auto,
            ..default()
        }));

        // Spawn instructions text
        parent.spawn(TextBundle::from_section(
            "Instructions:\nSpacebar => Shoot\nKill all enemies to level up\nRestore health with food",
            TextStyle {
                font_size: 30.0,
                color: Color::rgb(0.8, 0.8, 0.8),
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(250.0),
            left: Val::Auto,
            right: Val::Auto,
            ..default()
        }));

        // Spawn start prompt text
        parent.spawn(TextBundle::from_section(
            "Press S to Start",
            TextStyle {
                font_size: 40.0,
                color: Color::rgb(0.0, 1.0, 0.0), // Green color for emphasis
                ..default()
            },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(400.0),
            left: Val::Auto,
            right: Val::Auto,
            ..default()
        }));
    });
}

// Add this new system to handle the flash effect
fn update_damage_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut DamageFlash)>,
) {
    for (entity, mut flash) in flash_query.iter_mut() {
        flash.lifetime -= time.delta_seconds();
        if flash.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
