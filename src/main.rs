use bevy::{
    math::bounding::{Aabb2d, BoundingVolume},
    prelude::*,
    window::WindowResized,
};
use bevy_egui::{egui::Color32, prelude::*};
use bevy_rapier2d::prelude::*;
use bevy_svg::prelude::*;
use derive_more::{Constructor, IsVariant};
use rand::RngExt;

use crate::pascal_triangle::PascalTriangle;

mod pascal_triangle;

const BALL_RADIUS: f32 = 8.;
const BALL_RESTITUTION: Restitution = Restitution::coefficient(0.5);
const BALL_FRICTION: Friction = Friction::coefficient(0.05);
const BALL_SPAWN_JITTER: f32 = 0.05;
const BALL_TEXTURE_DIMS: f32 = 733.;

const PEG_RADIUS: f32 = 3.;
const PEG_RESTITUTION: Restitution = Restitution::coefficient(0.5);
const PEG_FRICTION: Friction = Friction::coefficient(0.05);
const PEG_COLOR: Color = Color::srgb_u8(0xFE, 0x91, 0xCA);

const PEG_HORIZONTAL_SPACING: f32 = 80.;
const PEG_VERTICAL_SPACING: f32 = 40.;
const PEG_SPAWN_JITTER: f32 = 0.05;

const WALL_RADIUS: f32 = 5.;
const WALL_RESTITUTION: Restitution = Restitution::coefficient(0.5);
const WALL_FRICTION: Friction = Friction::coefficient(0.075);
const WALL_COLOR: Color = Color::srgb_u8(0xA0, 0xA0, 0xA0);

const SPAWN_AREA_HEIGHT: f32 = 400.;
const MAX_SPAWN_AREA_BALLS: usize = 250;

const BOARD_CENTRERING_PADDING: f32 = 50.;

const BUCKET_LENGTH: f32 = 200.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Galton board simulation".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.))
        .add_plugins(SvgPlugin)
        .init_resource::<NumberOfBalls>()
        .init_resource::<PegLayers>()
        .init_resource::<RedrawBoard>()
        .init_resource::<SidePanelWidth>()
        .init_resource::<PascalTriangle>()
        .init_state::<SimState>()
        .add_systems(
            Startup,
            ((setup_camera_system, load_assets), setup_board).chain(),
        )
        .add_systems(PostStartup, update_camera_system.after(setup_camera_system))
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(
            Update,
            (
                (destroy_board, setup_board, (update_camera_system))
                    .chain()
                    .run_if(resource_equals::<RedrawBoard>(RedrawBoard(true))),
                update_camera_system
                    .run_if(resource_changed::<SidePanelWidth>.or(on_message::<WindowResized>)),
            ),
        )
        .add_systems(
            FixedUpdate,
            spawn_balls
                .before(PhysicsSet::StepSimulation)
                .run_if(in_state(SimState::Running).and(should_spawn_more)),
        )
        .add_systems(OnEnter(SimState::NotRunning), destroy_balls)
        .add_systems(
            OnEnter(SimState::Running),
            |mut commands: Commands, number_of_balls: Res<NumberOfBalls>| {
                commands.insert_resource(BallsToSpawn(number_of_balls.0));
            },
        )
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::IDENTITY));
}

fn update_camera_system(
    mut camera: Single<(&Camera, &mut Transform)>,
    board_bounding_box: Single<&BoardBoundingBox>,
    side_panel_width: Res<SidePanelWidth>,
) {
    let mut viewport_size = camera.0.logical_viewport_size().unwrap();
    viewport_size.x -= side_panel_width.0;
    let aabb = &board_bounding_box.0;

    let center = aabb.center();
    let size = aabb.max - aabb.min;
    let padded_size = size + Vec2::splat(BOARD_CENTRERING_PADDING * 2.0);

    camera.1.scale = Vec3::splat((padded_size / viewport_size).max_element().min(1000.));
    camera.1.translation.y = center.y;
}

#[derive(Debug, Component)]
struct BallMarker;

fn spawn_balls(
    mut commands: Commands,
    mut balls_to_spawn: ResMut<BallsToSpawn>,
    assets: Res<LoadedAssets>,
) {
    if balls_to_spawn.0 != 0 {
        let mut rng = rand::rng();
        let x_jitter = rng.random_range(-BALL_SPAWN_JITTER..=BALL_SPAWN_JITTER);
        let y_jitter = rng.random_range(-BALL_SPAWN_JITTER..=BALL_SPAWN_JITTER);

        let texture_scale = BALL_RADIUS / (BALL_TEXTURE_DIMS / 2.);
        commands.spawn((
            BallMarker,
            RigidBody::Dynamic,
            Collider::ball(BALL_RADIUS),
            Ccd::enabled(),
            BALL_RESTITUTION,
            BALL_FRICTION,
            Visibility::Visible,
            Transform::from_xyz(0. + x_jitter, 400. + y_jitter, 0.),
            children![(
                Svg2d(assets.ball.clone()),
                Transform::from_xyz(
                    -(BALL_TEXTURE_DIMS / 2. * texture_scale),
                    BALL_TEXTURE_DIMS / 2. * texture_scale,
                    1.,
                )
                .with_scale(Vec3::ONE * texture_scale),
            )],
        ));

        balls_to_spawn.0 -= 1;
    }
}

fn should_spawn_more(query: Query<&Transform, With<BallMarker>>) -> bool {
    let count = query
        .iter()
        .filter(|transform| transform.translation.truncate().y > 0.)
        .count();

    count < MAX_SPAWN_AREA_BALLS
}

#[derive(Resource, Default, PartialEq, Eq)]
struct RedrawBoard(bool);

#[derive(Resource, Default)]
struct SidePanelWidth(f32);

#[derive(Resource)]
struct PegLayers(u8);

impl Default for PegLayers {
    fn default() -> Self {
        Self(15)
    }
}

#[derive(Resource)]
struct NumberOfBalls(u32);

impl Default for NumberOfBalls {
    fn default() -> Self {
        Self(1000)
    }
}

#[derive(Resource)]
struct BallsToSpawn(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States, IsVariant)]
enum SimState {
    #[default]
    NotRunning,
    Running,
}

#[allow(
    clippy::too_many_arguments,
    reason = "Bevy requires this many to function"
)]
fn ui_system(
    mut redraw_board: ResMut<RedrawBoard>,
    mut peg_layers: ResMut<PegLayers>,
    mut number_of_balls: ResMut<NumberOfBalls>,
    mut side_panel_width: ResMut<SidePanelWidth>,
    mut sim_state_next: ResMut<NextState<SimState>>,
    sim_state: Res<State<SimState>>,
    mut contexts: EguiContexts,
    mut camera_transform: Single<&mut Transform, With<Camera>>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            // Move the camera off so its centered in the area remaining from the side panel
            let max_rect = ui.max_rect();
            let side_panel_width_ref = &side_panel_width;
            let new_side_panel_width = max_rect.max.x + max_rect.min.x;
            camera_transform.translation.x = new_side_panel_width * camera_transform.scale.x / -2.;
            if side_panel_width_ref.0 != new_side_panel_width {
                side_panel_width.0 = new_side_panel_width;
            }

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Galton board")
                        .color(Color32::WHITE)
                        .heading(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if sim_state.get().is_running() {
                        if ui.button("Cancel").clicked() {
                            sim_state_next.set(SimState::NotRunning);
                        };
                    } else {
                        if ui.button("Start").clicked() {
                            sim_state_next.set(SimState::Running);
                        };
                    }
                });
            });
            egui::Grid::new("settings").show(ui, |ui| {
                if sim_state.get().is_running() {
                    ui.disable();
                }

                // peg layers
                ui.label("Number of peg layers");
                redraw_board.0 = ui
                    .add(egui::Slider::new(&mut peg_layers.0, 10..=50).drag_value_speed(0.1))
                    .changed();
                ui.end_row();

                // number of balls
                ui.label("Number of balls");
                let old_number_of_balls = number_of_balls.0;
                ui.horizontal(|ui| {
                    if ui.button("-100").clicked() {
                        number_of_balls.0 = number_of_balls.0.saturating_sub(100);
                    }
                    if ui.button("-10").clicked() {
                        number_of_balls.0 = number_of_balls.0.saturating_sub(10);
                    }
                    if ui.button("-1").clicked() {
                        number_of_balls.0 = number_of_balls.0.saturating_sub(1);
                    }
                    ui.add(
                        egui::DragValue::new(&mut number_of_balls.0)
                            .speed(1)
                            .range(1..=10_000_000),
                    );
                    if ui.button("+1").clicked() {
                        number_of_balls.0 = number_of_balls.0.saturating_add(1);
                    }
                    if ui.button("+10").clicked() {
                        number_of_balls.0 = number_of_balls.0.saturating_add(10);
                    }
                    if ui.button("+100").clicked() {
                        number_of_balls.0 = number_of_balls.0.saturating_add(100);
                    }
                });
                if number_of_balls.0 != old_number_of_balls {
                    redraw_board.0 = true;
                }
                ui.end_row();
            });
        });
    }
}

#[derive(Component)]
struct Board;

#[derive(Component, Constructor)]
struct BoardBoundingBox(Aabb2d);

/// Convenience struct for generating multiple walls
#[derive(Constructor)]
struct WallBundleFactory<'a> {
    assets: &'a LoadedAssets,
    meshes: &'a mut Assets<Mesh>,
}

impl<'a> WallBundleFactory<'a> {
    fn wall(&mut self, a: Vec2, b: Vec2) -> WallBundle {
        WallBundle::new(a, b, self.assets, self.meshes)
    }
}

#[derive(Bundle)]
struct WallBundle {
    collider: Collider,
    friction: Friction,
    restitution: Restitution,
    ccd: Ccd,

    mesh: Mesh2d,
    material: MeshMaterial2d<ColorMaterial>,

    transform: Transform,
}

impl WallBundle {
    fn new(a: Vec2, b: Vec2, assets: &LoadedAssets, meshes: &mut Assets<Mesh>) -> Self {
        let delta = b - a;
        let length = delta.length();
        let midpoint = (a + b) * 0.5;

        let rotation = Quat::from_rotation_z(-delta.x.atan2(delta.y));
        let half = length * 0.5;

        Self {
            collider: Collider::capsule(Vec2::Y * half, Vec2::NEG_Y * half, WALL_RADIUS),
            friction: WALL_FRICTION,
            restitution: WALL_RESTITUTION,
            ccd: Ccd::enabled(),

            mesh: Mesh2d(meshes.add(Capsule2d::new(WALL_RADIUS, length))),
            material: MeshMaterial2d(assets.wall.clone()),

            transform: Transform {
                translation: midpoint.extend(0.0),
                rotation,
                ..default()
            },
        }
    }
}

// This whole function is a warcrime
fn setup_board(
    mut commands: Commands,
    peg_layers: Res<PegLayers>,
    number_of_balls: Res<NumberOfBalls>,
    assets: Res<LoadedAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut rng = rand::rng();
    let mut board_bounding_box = Aabb2d::new(Default::default(), Default::default());

    commands
        .spawn((Board, Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            let mut horizontal_offset_base = 0.;

            // Spawn pegs
            for layer in 0..peg_layers.0 {
                for i in 0..=layer {
                    let x_jitter = rng.random_range(-PEG_SPAWN_JITTER..=PEG_SPAWN_JITTER);
                    let y_jitter = rng.random_range(-PEG_SPAWN_JITTER..=PEG_SPAWN_JITTER);
                    parent.spawn((
                        Collider::ball(PEG_RADIUS),
                        PEG_RESTITUTION,
                        PEG_FRICTION,
                        Ccd::enabled(),
                        Transform::from_xyz(
                            x_jitter + horizontal_offset_base + (i as f32) * PEG_HORIZONTAL_SPACING,
                            y_jitter - (layer as f32) * PEG_VERTICAL_SPACING,
                            0.,
                        ),
                        Mesh2d(assets.peg_mesh.clone()),
                        MeshMaterial2d(assets.peg.clone()),
                    ));
                }
                horizontal_offset_base -= PEG_HORIZONTAL_SPACING / 2.;
            }

            let mut wall_factory = WallBundleFactory::new(&assets, &mut meshes);

            // Spawn bucket walls
            horizontal_offset_base -= PEG_HORIZONTAL_SPACING / 2.;
            let last_layer_y = -((peg_layers.0 - 1) as f32) * PEG_VERTICAL_SPACING;

            let n_balls = number_of_balls.0 as f32;
            let layers = peg_layers.0 as f32;

            let center_probability = (2.0 / (layers * std::f32::consts::PI)).sqrt();
            let center_count = n_balls * center_probability;

            let ball_volume = std::f32::consts::PI * BALL_RADIUS * BALL_RADIUS;
            let circle_packing = 1.0;
            let required_area = center_count * ball_volume / circle_packing;

            let bucket_width = PEG_HORIZONTAL_SPACING;
            let mut dynamic_bucket_length = BUCKET_LENGTH;

            if required_area > dynamic_bucket_length * bucket_width {
                dynamic_bucket_length = required_area / bucket_width * 1.15;
            }

            let bucket_floor_y = last_layer_y - dynamic_bucket_length;

            for i in 0..peg_layers.0 + 2 {
                parent.spawn(wall_factory.wall(
                    Vec2::new(
                        horizontal_offset_base + (i as f32) * PEG_HORIZONTAL_SPACING,
                        last_layer_y,
                    ),
                    Vec2::new(
                        horizontal_offset_base + (i as f32) * PEG_HORIZONTAL_SPACING,
                        bucket_floor_y,
                    ),
                ));
            }

            let bounding_box_bot_left = Vec2::new(
                horizontal_offset_base - WALL_RADIUS,
                bucket_floor_y - WALL_RADIUS,
            );

            // Spawn bucket floor
            let floor_left_point = Vec2::new(horizontal_offset_base, bucket_floor_y);
            parent.spawn(wall_factory.wall(floor_left_point, flip_x(floor_left_point)));

            // Spawn side barriers
            fn flip_x(v: Vec2) -> Vec2 {
                v * Vec2::new(-1.0, 1.0)
            }

            let point1 = Vec2::from((horizontal_offset_base, last_layer_y));
            let point2 = Vec2::from((-PEG_HORIZONTAL_SPACING / 2., PEG_VERTICAL_SPACING));
            parent.spawn(wall_factory.wall(point1, point2));
            parent.spawn(wall_factory.wall(flip_x(point1), flip_x(point2)));

            // Spawn the spawn area floor
            let bucket_point = Vec2::from((-300., 175.));
            parent.spawn(wall_factory.wall(point2, bucket_point));
            parent.spawn(wall_factory.wall(flip_x(point2), flip_x(bucket_point)));

            // Spawn the spawn area walls
            let bucket_ceil_point1 = Vec2 {
                x: bucket_point.x,
                y: bucket_point.y + SPAWN_AREA_HEIGHT,
            };
            let bucket_ceil_point2 = flip_x(bucket_ceil_point1);
            parent.spawn(wall_factory.wall(bucket_point, bucket_ceil_point1));
            parent.spawn(wall_factory.wall(flip_x(bucket_point), bucket_ceil_point2));

            // Spawn the spawn area ceiling
            parent.spawn(wall_factory.wall(bucket_ceil_point1, bucket_ceil_point2));

            let bounding_box_top_right =
                Vec2::new(-bounding_box_bot_left.x, bucket_ceil_point1.y + WALL_RADIUS);

            board_bounding_box = Aabb2d {
                min: bounding_box_bot_left,
                max: bounding_box_top_right,
            };
        })
        .insert(BoardBoundingBox::new(board_bounding_box));
}

fn destroy_board(mut commands: Commands, query: Single<Entity, With<Board>>) {
    commands.entity(*query).despawn();
}

fn destroy_balls(mut commands: Commands, query: Query<Entity, With<BallMarker>>) {
    for ball in query {
        commands.entity(ball).despawn();
    }
}

#[derive(Resource)]
struct LoadedAssets {
    ball: Handle<Svg>,
    peg: Handle<ColorMaterial>,
    wall: Handle<ColorMaterial>,

    peg_mesh: Handle<Mesh>,
}

fn load_assets(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ball = server.load("ball.svg");
    let peg = materials.add(ColorMaterial::from_color(PEG_COLOR));
    let wall = materials.add(ColorMaterial::from_color(WALL_COLOR));
    let peg_mesh = meshes.add(Circle::new(PEG_RADIUS));
    commands.insert_resource(LoadedAssets {
        ball,
        peg,
        wall,
        peg_mesh,
    });
}
