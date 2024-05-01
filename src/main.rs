use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

#[derive(Component, Debug, Default, Clone)]
struct Cell {
    shape: Rectangle,
    transform: Transform,
    entity: Option<Entity>
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
}

impl Grid {
    fn new(w: f32, h: f32, thick: f32, size: f32) -> Grid {
        if w % size != 0.0 {
            panic!("The screen width must be evenly divided by the side of the cell!");
        }
        if h % size != 0.0 {
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
                        0.0 + (x as i32 * size as i32) as f32 - size / 2.0 - w / 2.0,
                        0.0 + (y as i32 * size as i32) as f32 - size / 2.0 - h / 2.0,
                        0.0,
                    ),
                    entity: None,
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
}

fn main() {
    let w = 1200.0;
    let h = 960.0;
    let title = String::from("The Game of Life");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title,
                resizable: false,
                resolution: (w, h).into(),
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
    let w = 1200.0;
    let h = 960.0;
    let grid = Grid::new(w, h, 1.0, 24.0);
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
    commands.spawn(grid);
}

fn render_grid(
    mut grid_q: Query<&mut Grid>,
    cells_q: Query<(Entity, &Cell)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut grid = match grid_q.get_single_mut() {
        Ok(value) => value,
        Err(val) => {
            println!("{}", val);
            return;
        }
    };
    if input.just_pressed(KeyCode::KeyP) || input.just_pressed(KeyCode::Space) {
        grid.frozen = !grid.frozen;
    }
    if !grid.frozen {
        for (e, cell) in cells_q.iter() {
            // delete the old cells
            commands.entity(e).despawn();
        }
        let mut data = grid.data.clone();
        for (x, row) in data.iter_mut().enumerate() {
            for (y, cell) in row.iter_mut().enumerate() {
                commands.spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(cell.shape)),
                    material: materials.add(Color::BLACK),
                    transform: cell.transform,
                    ..default()
                });
            }
        }
    }
}
