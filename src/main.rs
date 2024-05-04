use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

/// Field refresh rate in seconds
const REFRESH_RATE: f32 = 0.05;
const CELL_SIZE: f32 = 18.0;
// Height and width of the window
const WIDTH: f32 = CELL_SIZE * 80.0;
const HEIGHT: f32 = CELL_SIZE * 45.0;

#[derive(Component, Debug, Default, Clone)]
struct Cell {
    shape: Rectangle,
    transform: Transform,
    active: bool,
    x: usize,
    y: usize,
}

struct Lines {
    shape: Rectangle,
    transform: Transform,
}

#[derive(Component)]
struct Grid {
    data: Vec<Vec<Cell>>,
    size: f32,
    thick: f32,
    w: f32,
    h: f32,
    columns: i32,
    rows: i32,
    frozen: bool,
    timer: Timer,
}

impl Grid {
    fn new(w: f32, h: f32, thick: f32, size: f32) -> Grid {
        if w % size != 0.0 || size <= 1.0 {
            panic!("The screen WIDTH must be evenly divided by the side of the cell!");
        }
        if h % size != 0.0 || size <= 1.0 {
            panic!("The screen height must be evenly divided by the side of the cell!");
        }
        let columns = (w / size) as usize;
        let rows = (h / size) as usize;
        let mut data = vec![vec![Cell::default(); rows]; columns];
        for x in 0..columns {
            for y in 0..rows {
                data[x][y] = Cell {
                    shape: Rectangle::new(size, size),
                    transform: Transform::from_xyz(
                        0.0 + (x as i32 * size as i32) as f32 - size / 2.0 - w / 2.0 + size,
                        0.0 + (y as i32 * size as i32) as f32 - size / 2.0 - h / 2.0 + size,
                        -1.0,
                    ),
                    active: false,
                    x,
                    y: rows - y - 1,
                };
            }
        }
        Grid {
            data,
            size,
            thick,
            w,
            h,
            columns: columns as i32,
            rows: rows as i32,
            frozen: true,
            timer: Timer::from_seconds(REFRESH_RATE, TimerMode::Repeating),
        }
    }
    fn get_meshes(&self) -> Vec<Lines> {
        let mut meshes = vec![];
        for i in 0..self.columns {
            meshes.push(Lines {
                shape: Rectangle::new(self.thick, self.h),
                transform: Transform::from_xyz(
                    (0.0 + i as f32 * self.size) - self.w / 2.0,
                    0.0,
                    0.0,
                ),
            });
        }
        for i in 0..self.rows {
            meshes.push(Lines {
                shape: Rectangle::new(self.w, self.thick),
                transform: Transform::from_xyz(
                    0.0,
                    (0.0 + i as f32 * self.size) - self.h / 2.0,
                    0.0,
                ),
            });
        }
        meshes
    }

    fn count_active_neighbours(&self, x: usize, y: usize) -> i32 {
        let mut count = 0;
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue;
                }
                let nx = ((x as i32 + i) + self.columns) % self.columns;
                let ny = ((y as i32 + j) + self.rows) % self.rows;
                if self.data[nx as usize][ny as usize].active {
                    count += 1;
                }
            }
        }
        count
    }
}

fn main() {
    let title = String::from("The Game of Life");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title,
                resizable: false,
                resolution: (WIDTH, HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, render_grid)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut grid = Grid::new(WIDTH, HEIGHT, 1.0, CELL_SIZE);
    let grid_meshes = grid.get_meshes();
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
        ..default()
    });
    for mesh in grid_meshes {
        commands.spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(mesh.shape)),
            material: materials.add(Color::GRAY),
            transform: mesh.transform,
            ..default()
        });
    }
    for row in grid.data.iter_mut() {
        for cell in row.iter_mut() {
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(cell.shape)),
                        material: materials.add(Color::BLACK),
                        transform: cell.transform,
                        ..default()
                    },
                    cell.clone(),
                ))
                .insert(Visibility::Hidden);
        }
    }
    commands.spawn(grid);
}

fn render_grid(
    time: Res<Time>,
    mut grid_q: Query<&mut Grid>,
    mut cell_query: Query<(Entity, &mut Cell)>,
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    input: Res<ButtonInput<KeyCode>>,
    window_q: Query<&mut Window>,
) {
    let mut grid = grid_q.get_single_mut().unwrap();
    let window = window_q.get_single().unwrap();
    let mut change_to_active = None;
    grid.timer.tick(time.delta());

    // Changing the state of the cell when pressed
    if mouse.pressed(MouseButton::Left) {
        change_to_active = Some(true);
    } else if mouse.pressed(MouseButton::Right) {
        change_to_active = Some(false);
    }

    // Defrost or freeze time
    if input.just_pressed(KeyCode::KeyP)
        || input.just_pressed(KeyCode::Space)
        || input.just_pressed(KeyCode::Enter)
    {
        grid.frozen = !grid.frozen;
    }

    // Erase all the cells
    if input.just_pressed(KeyCode::KeyE)
        || input.just_pressed(KeyCode::Backspace)
        || input.just_pressed(KeyCode::Delete)
    {
        for (e, mut cell) in cell_query.iter_mut() {
            commands.entity(e).insert(Visibility::Hidden);
            cell.active = false;
            grid.data[cell.x][cell.y].active = false;
        }
    }

    if !grid.frozen && grid.timer.finished() {
        // probing and rendering a new generation of cells
        let mut new_generation = vec![vec![false; grid.rows as usize]; grid.columns as usize];
        for (e, mut cell) in cell_query.iter_mut() {
            let count = grid.count_active_neighbours(cell.x, cell.y);
            if cell.active && (count == 2 || count == 3) {
                // the cell stays alive
                commands.entity(e).insert(Visibility::Visible);
                new_generation[cell.x][cell.y] = true;
                cell.active = true;
            } else if !cell.active && count == 3 {
                // a dead cell with exactly 3 neighbours becomes alive
                commands.entity(e).insert(Visibility::Visible);
                new_generation[cell.x][cell.y] = true;
                cell.active = true;
            } else {
                // the cell dies
                commands.entity(e).insert(Visibility::Hidden);
                new_generation[cell.x][cell.y] = false;
                cell.active = false;
            }
        }
        for (x, row) in new_generation.iter().enumerate() {
            for (y, active) in row.iter().enumerate() {
                grid.data[x][y].active = *active;
            }
        }
        grid.timer.reset();
    }

    match change_to_active {
        Some(value) => {
            let position = match window.cursor_position() {
                None => return,
                Some(value) => value,
            };
            let cell_x = position.x / grid.size;
            let cell_y = position.y / grid.size;

            let cell_x = cell_x.floor() as usize;
            let cell_y = cell_y.floor() as usize;

            if cell_x < grid.columns as usize && cell_y < grid.rows as usize {
                for (e, mut cell) in cell_query.iter_mut() {
                    if cell.x == cell_x && cell.y == cell_y {
                        match value {
                            true => {
                                commands.entity(e).insert(Visibility::Visible);
                                grid.data[cell.x][cell.y].active = true;
                                cell.active = true;
                            }
                            false => {
                                commands.entity(e).insert(Visibility::Hidden);
                                grid.data[cell.x][cell.y].active = false;
                                cell.active = false;
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }
}
