//! Seems like space invaders classic, then suddenly the aliens fly out of their formation and attack!
//! Then suddenly, you realize you are not constrained to the bottom of the screen,
//! you can go all around, then you can dive through the middle.  You barriers morph into something fun.
//! Then you realize you can fly outside the screen as the little spot where you started shrinks in the distance
//! and there's all kinds of havoc going on in neigboring areas.
//!
//! Alternatively, starts the same classic space invaders, then same morphs into Galaxion,
//! then you fly forward and start pikcing up people as in Defender.  Oh no! asteroids start flying toward you!
//! Oh no, aliens are mining the little asteroids and creating a huge demonic boss, don't let them! Sinistar!

use bevy::{prelude::*, render::camera::Camera2d,
    math::{const_vec3, const_vec2},
    sprite::collide_aabb::{collide},};
use rand::{thread_rng, Rng};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;
const WIN_WIDTH:f32 = 1600.0;
const WIN_HEIGHT:f32 = 900.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(BoltExists(false))
        .insert_resource(ClassicMarch{direction:MarchDir::Right})
        .insert_resource(WindowDescriptor {
            title: "Angry Blinky Creatures from Space!".to_string(),
            width:WIN_WIDTH,
            height:WIN_HEIGHT,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(animate_sprite)
        .add_event::<CollisionEvent>() // explosion catches this
        .add_system(explosion)
        .add_system(check_for_collisions) 
        .add_system(move_alien_classic)
        .add_system(move_alien_circle)
        .add_system(move_alien_attack)
        .add_system(move_player)
        .add_system(move_bolt)
        .add_system(player_camera_control)
        .run();
}

#[derive(Default)]
struct CollisionEvent {
    pos:Vec3
}

#[derive(Clone,Copy)]
enum MarchDir {
    Left,
    Right,
    //Down,

}
#[derive(Clone,Copy)]
struct ClassicMarch {
    direction: MarchDir,
}

impl ClassicMarch {
    fn vec3(self) -> Vec3 {
        match self.direction {
            MarchDir::Left => Vec3::new(-1.0, 0.0, 0.0),
            MarchDir::Right => Vec3::new(1.0, 0.0, 0.0),
            //MarchDir::Down => Vec3::new(0.0, -1.0, 0.0),
        }
    }
}
#[derive(Component)]
/// classic space invaders rank and file
struct Classic; 

#[derive(Component)]
/// target the player
struct Attack;  

#[derive(Component)]
/// circling around the screen
struct Circle;  

#[derive(Component)]
struct AlienMoves {
    speed: Vec2,
    target: Option<Vec2>,
}

impl AlienMoves {
    fn new() -> Self {
        Self {
            //mood: MoveMood::Classic,
            speed: rnd_vec2(),
            target: None,
        }
    }
}
fn rnd_vec2() -> Vec2 {
    Vec2::new(
        thread_rng().gen_range(-1.0..1.0),
        thread_rng().gen_range(-1.0..1.0),
    )
}

/// next point for a circle
/// https://www.quora.com/How-can-you-write-code-to-draw-a-circle-without-using-sine-cosine-or-sqrt
fn circle_mut( v: &mut Vec2, r:Vec2) {
    let (mut x, mut y) = (v.x - r.x, v.y - r.y);
    x += y*0.01;
    y -= x*0.01;
    *v = Vec2::new( x + r.x,  y + r.y);
}


/// System to handle classic marching aliens
fn move_alien_classic(    
        mut commands: Commands,
        mut classic_march: ResMut<ClassicMarch>, 
        mut query: Query<(Entity, &mut Transform), With<Classic>>
    ) {
    use MarchDir::*;
    let mut rng = thread_rng();
    for (entity, mut transform) in query.iter_mut() {
        if transform.translation.x >= WIN_WIDTH/2.0{
            classic_march.direction = Left;
        } else if transform.translation.x <= -WIN_WIDTH/2.0{
            classic_march.direction = Right;
        }
        transform.translation += classic_march.vec3();
        match rng.gen_range(0..=10000i32) {
            //1 => {
            //    commands.entity(entity).remove::<Classic>();
            //    commands.entity(entity).insert(Attack);
            //}
            2 => {                
                commands.entity(entity).remove::<Classic>();
                commands.entity(entity).insert(Circle);
            }
            _ => {}
        }
    }
}

fn move_alien_attack(mut query: Query<(&AlienMoves, &mut Transform), With<Attack>>) {
    for (moovy, mut transform) in query.iter_mut() {
                transform.translation += moovy.speed.extend(0.0)
    }
}

fn move_alien_circle(mut query: Query<(&mut AlienMoves, &mut Transform),With<Circle>>) {
    //let rng = thread_rng();
    for (mut moovy, mut transform) in query.iter_mut() {
        //match rng.gen_range(0..=1000i32) {
        //    1=> {moovy.speed *= Vec2::new(-1.,1.)},
        //    2=> {moovy.speed *= Vec2::new(1.,-1.)},
        //    3=> {moovy.speed *= Vec2::new(-1.,-1.)},
        //    _=>{}
        //}
        let center = moovy.target.unwrap_or(Vec2::new(0.0,0.0));
        circle_mut(&mut moovy.speed, center);
        transform.translation += moovy.speed.extend(0.0)
    }
}



#[derive(Component)]
struct AnimationTimer {
    timer: Timer,
    // number of images from the spritesheet
    pub frames: u32,
    // 0 is the first image in the spritesheet
    // animation will start at start_index, and end at start_index + frames-1
    pub start_index: u32,
    pub repeat : bool,
}

fn animate_sprite(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite, Entity)>,
) {
    for (mut anime, mut sprite, entity) in query.iter_mut() {
        anime.timer.tick(time.delta());
        if anime.timer.just_finished() {
            if ! anime.repeat && anime.frames == sprite.index as u32 - anime.start_index + 1 {
                commands.entity(entity).despawn();
            }
            //let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (((sprite.index - anime.start_index as usize) + 1)
                % anime.frames as usize)
                + anime.start_index as usize;
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

const EXPLOSION_SIZE:f32 = 8.0;
const BOLT_COLOR: Color = Color::rgb(1.0, 0.5, 0.0);
const PLAYER_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const PLAYER_Y:f32 = -WIN_HEIGHT/2.0 + 100.0;
// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
// The `const_vec3!` macros are needed as functions that operate on floats cannot be constant in Rust.
const PLAYER_SIZE: Vec3 = const_vec3!([120.0, 20.0, 0.0]);
const PLAYER_SPEED: f32 = 500.0;

/// User controls his ship
fn move_player(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    bolt_exists: ResMut<BoltExists>,
    atlases: Res<Atlases>,
    //mut texture_atlases: ResMut<Assets<TextureAtlas>>,  //////TODO
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    // Calculate the new horizontal player position based on player input
    let new_player_position = player_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;

    // Update the player position,
    // making sure it doesn't cause the player to leave the arena
    let left_bound = -WIN_WIDTH / 2.0 + PLAYER_SIZE.x / 2.0;
    let right_bound = WIN_WIDTH / 2.0 - PLAYER_SIZE.x / 2.0;

    player_transform.translation.x = new_player_position.clamp(left_bound, right_bound);
    if let BoltExists(false)= *bolt_exists {
        if keyboard_input.just_pressed(KeyCode::Space) 
        {
            //*bolt_exists = BoltExists(true);// 👾
            commands.spawn().insert_bundle(SpriteSheetBundle {
                texture_atlas: atlases.bolt.clone(),
                transform: Transform { 
                    translation: player_transform.translation,
                    scale: BOLT_SIZE,
                    ..default()
                },
                sprite: TextureAtlasSprite {index:0, color: BOLT_COLOR, ..Default::default() },
                ..Default::default()
            })
            .insert(Bolt)
            .insert(Velocity(INITIAL_BOLT_DIRECTION.normalize() * BOLT_SPEED))
            .insert(AnimationTimer {
                timer: Timer::from_seconds(0.1, true),
                frames: 5,
                start_index: 0,
                repeat: true,
            });
        }
    }

}


// We set the z-value of the bolt to 1 so it renders on top in the case of overlapping sprites.
const BOLT_SIZE: Vec3 = const_vec3!([2.0, 2.0, 1.0]);
const BOLT_SPEED: f32 = 600.0;
const INITIAL_BOLT_DIRECTION: Vec2 = const_vec2!([0.0, 1.0]);

#[derive(Component)]
struct Bolt;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Deref, DerefMut)]
struct BoltExists(bool);

/// Mostly for bullets
fn move_bolt(
    mut commands: Commands,
    mut bolt_exists: ResMut<BoltExists>,
    mut query: Query<(Entity, &mut Transform, &Velocity)>) {
    for (entity, mut transform, velocity) in query.iter_mut() {
        if transform.translation.x.abs() >= WIN_WIDTH / 2.0 
            || transform.translation.y.abs() >= WIN_HEIGHT / 2.0 {
            commands.entity(entity).despawn();
            *bolt_exists = BoltExists(false);
            return;
        }
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn check_for_collisions(
    mut commands: Commands,
    bolt_query: Query<&Transform, With<Bolt>>,
    collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    for bolt_transform in bolt_query.iter() {
        let bolt_size = bolt_transform.scale.truncate();

        // check collision with walls
        for (collider_entity, transform) in collider_query.iter() {
            let collision = collide(
                bolt_transform.translation,
                bolt_size,
                transform.translation,
                transform.scale.truncate() * 8.0,
            );
            if let Some(_/*collision*/) = collision {
                // Sends a collision event so that other systems can react to the collision
                collision_events.send(CollisionEvent { pos: transform.translation });
                commands.entity(collider_entity).despawn();
            }
        }
    }
}

/// event handler when collisions happen.
fn explosion(
    mut commands: Commands,
    atlases: Res<Atlases>,
    mut event_reader: EventReader<CollisionEvent>,
)  {
    for collision_event in event_reader.iter() {
        let pos = collision_event.pos;
        commands.spawn().insert_bundle(
            SpriteSheetBundle {
                texture_atlas: atlases.explosion.clone(),
                transform: Transform { 
                    translation: pos,
                    scale: Vec3::splat(EXPLOSION_SIZE),
                    ..default()
                },
                sprite: TextureAtlasSprite { 
                    color: Color::rgb(1.0, 1.0, 0.0), 
                    index: 0,
                    ..Default::default() 
                },
                ..Default::default()
            })        //.insert(Collider)
        .insert(AnimationTimer {
            timer: Timer::from_seconds(0.1, true),
            frames: 4,
            start_index: 0,
            repeat: false,
        });
    }
}

struct Atlases {
    bolt : Handle<TextureAtlas>,
    explosion : Handle<TextureAtlas>,
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("aliens.png"); //"gabe-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(12.0, 12.0), 4, 1);
    let alien_atlas_h = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load("bolt.png"); 
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(7.0, 12.0), 5, 1);
    let bolt_atlas_h = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load("explosion_bl.png"); 
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(12.0, 12.0), 4, 1);
    let expl_atlas_h = texture_atlases.add(texture_atlas);


    commands.insert_resource(Atlases{bolt:bolt_atlas_h, explosion:expl_atlas_h});

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());


    // Player's ship
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, PLAYER_Y, 0.0),
                scale: PLAYER_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..default()
            },
            ..default()
        })
        //.insert(Collider) // todo bolt hits player on spawn, start it above or something.
        ;
    // Big alien
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: alien_atlas_h.clone(),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        })
        .insert(AnimationTimer {
            timer: Timer::from_seconds(0.5, true),
            frames: 2,
            start_index: 0,
            repeat: true,
        })
        .insert(AlienMoves::new())
        .insert(Classic)
        .insert(Collider);

    //make a bunch

    for row in 0..5 {
        for column in 0..20 {
            spawn_alien(
                &mut commands,
                alien_atlas_h.clone(),
                2,
                if (column + row) % 2 == 0 { 2 } else { 0 },
                0.1 + column as f32 / 40.0,
                Vec3::new((column * 50 - 500) as f32, (-row * 50 + 400) as f32, 0.0),
            );
        }
    }
}

fn spawn_alien(
    commands: &mut Commands,
    atlas_h: Handle<TextureAtlas>,
    frames: u32,
    start_index: u32,
    duration: f32,
    position: Vec3,
) {
    let r = (position.x.abs() / 501.0).clamp(0.0, 1.0);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas_h,
            transform: Transform {
                translation: position,
                scale: Vec3::splat(thread_rng().gen_range(2..=4) as f32),//3.0),
                ..Default::default()
            },
            sprite: TextureAtlasSprite {
                index: 2,
                color: Color::rgb(r, 0.0, 1.0 - r),
                ..Default::default()
            },
            ..default()
        })
        .insert(AnimationTimer {
            timer: Timer::from_seconds(duration, true),
            frames,
            start_index,
            repeat: true,
        })
        ////////////// Movement is setup here
        .insert(AlienMoves::new())
        .insert(Classic)
        .insert(Collider);
}

/// zoom in and out
fn player_camera_control(
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    const CAMERA_SPEED_PER_SEC: f32 = 1.0;
    let dist = CAMERA_SPEED_PER_SEC * time.delta().as_secs_f32();

    for mut projection in query.iter_mut() {
        let mut log_scale = projection.scale.ln();

        if kb.pressed(KeyCode::PageUp) {
            log_scale -= dist;
        }
        if kb.pressed(KeyCode::PageDown) {
            log_scale += dist;
        }

        projection.scale = log_scale.exp();
    }
}

