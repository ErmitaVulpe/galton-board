use bevy::prelude::*;
use bevy_egui::{egui::Color32, prelude::*};
use bevy_rapier2d::prelude::*;
use derive_more::IsVariant;
use rand::RngExt;

const BALL_RADIUS: f32 = 5.;
const BALL_RESTITUTION: Restitution = Restitution::coefficient(0.2);
const BALL_FRICTION: Friction = Friction::coefficient(1.5);
const BALL_SPAWN_JITTER: f32 = 0.05;

const PEG_RADIUS: f32 = 10.;
const PEG_RESTITUTION: Restitution = Restitution::coefficient(0.5);
const PEG_FRICTION: Friction = Friction::coefficient(0.7);

const WALL_RADIUS: f32 = PEG_RADIUS / 2.;
const WALL_RESTITUTION: Restitution = Restitution::coefficient(1.);
const WALL_FRICTION: Friction = Friction::coefficient(100.);

const PEG_HORIZONTAL_SPACING: f32 = 80.;
const PEG_VERTICAL_SPACING: f32 = 40.;
const PEG_SPAWN_JITTER: f32 = 0.05;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.))
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<RedrawBoard>()
        .init_resource::<PegLayers>()
        .init_resource::<NumberOfBalls>()
        .init_state::<SimState>()
        .add_systems(Startup, (setup_camera_system, setup_board))
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(
            Update,
            ((destroy_board, setup_board)
                .chain()
                .run_if(resource_equals::<RedrawBoard>(RedrawBoard(true))),),
        )
        .add_systems(FixedUpdate, spawn_balls.run_if(in_state(SimState::Running)))
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
    commands.spawn((Camera2d, Transform::from_scale(Vec3::ONE * 3.)));
}

#[derive(Debug, Component)]
struct BallMarker;

fn spawn_balls(mut commands: Commands, mut balls_to_spawn: ResMut<BallsToSpawn>) {
    if balls_to_spawn.0 != 0 {
        let mut rng = rand::rng();
        let x_jitter = rng.random_range(-BALL_SPAWN_JITTER..=BALL_SPAWN_JITTER);
        let y_jitter = rng.random_range(-BALL_SPAWN_JITTER..=BALL_SPAWN_JITTER);

        commands.spawn((
            BallMarker,
            RigidBody::Dynamic,
            Collider::ball(BALL_RADIUS),
            Ccd::enabled(),
            BALL_RESTITUTION,
            BALL_FRICTION,
            Transform::from_xyz(0. + x_jitter, 400. + y_jitter, 0.),
        ));

        balls_to_spawn.0 -= 1;
    }
}

#[derive(Resource, Default, PartialEq, Eq)]
struct RedrawBoard(bool);

#[derive(Resource)]
struct PegLayers(u8);

impl Default for PegLayers {
    fn default() -> Self {
        Self(7)
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

fn ui_system(
    mut redraw_board: ResMut<RedrawBoard>,
    mut peg_layers: ResMut<PegLayers>,
    mut number_of_balls: ResMut<NumberOfBalls>,
    mut sim_state_next: ResMut<NextState<SimState>>,
    sim_state: Res<State<SimState>>,
    mut contexts: EguiContexts,
    mut camera_transform: Single<&mut Transform, With<Camera>>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
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
                        .add(egui::Slider::new(&mut peg_layers.0, 3..=30).drag_value_speed(0.1))
                        .changed();
                    ui.end_row();

                    // number of balls
                    ui.label("Number of balls");
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
                        ui.add(egui::DragValue::new(&mut number_of_balls.0).speed(1));
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
                    ui.end_row();
                });

                // Move the camera off so its centered in the area remaining from the side panel
                camera_transform.translation.x =
                    ui.available_width() * camera_transform.scale.x / -2.;
            });
    }
}

#[derive(Component)]
struct Board;

#[derive(Debug, Bundle)]
struct WallBundle {
    collider: Collider,
    friction: Friction,
    restitution: Restitution,
    ccd: Ccd,
}

impl WallBundle {
    fn new(point1: Vec2, point2: Vec2) -> Self {
        Self {
            collider: Collider::capsule(point1, point2, WALL_RADIUS),
            friction: WALL_FRICTION,
            restitution: WALL_RESTITUTION,
            ccd: Ccd::enabled(),
        }
    }
}

fn setup_board(mut commands: Commands, peg_layers: Res<PegLayers>) {
    let mut rng = rand::rng();

    commands
        .spawn((Board, Transform::default()))
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
                    ));
                }
                horizontal_offset_base -= PEG_HORIZONTAL_SPACING / 2.;
            }

            // Spawn bucket walls
            horizontal_offset_base -= PEG_HORIZONTAL_SPACING / 2.;
            let last_layer_y = -((peg_layers.0 - 1) as f32) * PEG_VERTICAL_SPACING;
            for i in 0..peg_layers.0 + 2 {
                parent.spawn(WallBundle::new(
                    Vec2::new(
                        horizontal_offset_base + (i as f32) * PEG_HORIZONTAL_SPACING,
                        last_layer_y,
                    ),
                    Vec2::new(
                        horizontal_offset_base + (i as f32) * PEG_HORIZONTAL_SPACING,
                        last_layer_y - 400., // TODO parametrise this bih
                    ),
                ));
            }

            // Spawn bucket floor
            let floor_left_point = Vec2::new(horizontal_offset_base, last_layer_y - 400.);
            parent.spawn(WallBundle::new(floor_left_point, flip_x(floor_left_point)));

            // Spawn side barriers
            fn flip_x(v: Vec2) -> Vec2 {
                v * Vec2::new(-1.0, 1.0)
            }

            let point1 = Vec2::from((horizontal_offset_base, last_layer_y));
            let point2 = Vec2::from((-PEG_HORIZONTAL_SPACING / 2., PEG_VERTICAL_SPACING));
            parent.spawn(WallBundle::new(point1, point2));
            parent.spawn(WallBundle::new(flip_x(point1), flip_x(point2)));

            // Spawn the spawn area
            let bucket_point = Vec2::from((-300., 175.));
            parent.spawn(WallBundle::new(point2, bucket_point));
            parent.spawn(WallBundle::new(flip_x(point2), flip_x(bucket_point)));
        });
}

fn destroy_board(mut commands: Commands, query: Single<Entity, With<Board>>) {
    commands.entity(*query).despawn();
}

fn destroy_balls(mut commands: Commands, query: Query<Entity, With<BallMarker>>) {
    for ball in query {
        commands.entity(ball).despawn();
    }
}
