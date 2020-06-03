// Standard library
use std::sync::Arc;

// External libraries
use cascade::cascade;
use crossterm::style::{style, Attribute, Color, StyledContent};
use vulkano::descriptor::pipeline_layout::{PipelineLayout, PipelineLayoutAbstract};
use vulkano::device::Device;
use vulkano::pipeline::ComputePipeline;

// CELL
use crate::automaton::*;
use crate::grid::{Dimensions, Grid, GridView, Position, MOORE_NEIGHBORHOOD};

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash, std::fmt::Debug)]
pub enum GameOfLife {
    Dead,
    Alive,
}

impl Default for GameOfLife {
    fn default() -> Self {
        Self::Dead
    }
}

impl Cell for GameOfLife {}

impl TranscodableCell for GameOfLife {
    fn encode(&self) -> u32 {
        match self {
            GameOfLife::Dead => 0,
            GameOfLife::Alive => 1,
        }
    }

    fn decode(id: u32) -> Self {
        match id {
            0 => GameOfLife::Dead,
            1 => GameOfLife::Alive,
            _ => panic!(format!("Decoding failed: unkwnon encoding {}.", id)),
        }
    }
}

impl UpdateCPU for GameOfLife {
    fn update_cell<'a>(grid: &GridView<'a, Self>) -> Self {
        // Count the number of alive cells around us
        let nb_alive_neighbors =
            grid.get_relative_mul(&MOORE_NEIGHBORHOOD)
                .iter()
                .fold(0, |cnt, cell| {
                    if let GameOfLife::Alive = cell {
                        cnt + 1
                    } else {
                        cnt
                    }
                });

        // Apply the evolution rule
        match grid.cell() {
            GameOfLife::Dead => {
                if nb_alive_neighbors == 3 {
                    GameOfLife::Alive
                } else {
                    GameOfLife::Dead
                }
            }
            GameOfLife::Alive => {
                if nb_alive_neighbors == 2 || nb_alive_neighbors == 3 {
                    GameOfLife::Alive
                } else {
                    GameOfLife::Dead
                }
            }
        }
    }
}

impl UpdateGPU for GameOfLife {
    type Pipeline = ComputePipeline<PipelineLayout<shader::Layout>>;
    type PushConstants = shader::ty::Dim;

    fn vk_setup(device: &Arc<Device>) -> PipelineInfo<Self::Pipeline> {
        let shader = shader::Shader::load(device.clone()).unwrap();
        let pipeline =
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap();
        let layout = pipeline.layout().descriptor_set_layout(0).unwrap().clone();
        PipelineInfo {
            layout,
            pipeline: Arc::new(pipeline),
        }
    }

    fn push_constants(grid: &Grid<Self>) -> Self::PushConstants {
        let dim = grid.dim();
        shader::ty::Dim {
            nb_rows: dim.height() as u32,
            nb_cols: dim.width() as u32,
        }
    }
}

impl TermDrawableAutomaton for GameOfLife {
    fn style(&self) -> StyledContent<char> {
        match self {
            GameOfLife::Dead => style('·').with(Color::Grey),
            GameOfLife::Alive => style('#').with(Color::Green).attribute(Attribute::Bold),
        }
    }
}

mod shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "game_of_life.comp",
    }
}

pub fn gosper_glider_gun() -> Grid<GameOfLife> {
    let mut grid = Grid::new(Dimensions::new(100, 50));
    grid = cascade!(
        grid;
        ..set(Position::new(1, 5), GameOfLife::Alive);
        ..set(Position::new(1, 6), GameOfLife::Alive);
        ..set(Position::new(2, 5), GameOfLife::Alive);
        ..set(Position::new(2, 6), GameOfLife::Alive);
        ..set(Position::new(11, 5), GameOfLife::Alive);
        ..set(Position::new(11, 6), GameOfLife::Alive);
        ..set(Position::new(11, 7), GameOfLife::Alive);
        ..set(Position::new(12, 4), GameOfLife::Alive);
        ..set(Position::new(12, 8), GameOfLife::Alive);
        ..set(Position::new(13, 3), GameOfLife::Alive);
        ..set(Position::new(13, 9), GameOfLife::Alive);
        ..set(Position::new(14, 3), GameOfLife::Alive);
        ..set(Position::new(14, 9), GameOfLife::Alive);
        ..set(Position::new(15, 6), GameOfLife::Alive);
        ..set(Position::new(16, 4), GameOfLife::Alive);
        ..set(Position::new(16, 8), GameOfLife::Alive);
        ..set(Position::new(17, 5), GameOfLife::Alive);
        ..set(Position::new(17, 6), GameOfLife::Alive);
        ..set(Position::new(17, 7), GameOfLife::Alive);
        ..set(Position::new(18, 6), GameOfLife::Alive);
        ..set(Position::new(21, 3), GameOfLife::Alive);
        ..set(Position::new(21, 4), GameOfLife::Alive);
        ..set(Position::new(21, 5), GameOfLife::Alive);
        ..set(Position::new(22, 3), GameOfLife::Alive);
        ..set(Position::new(22, 4), GameOfLife::Alive);
        ..set(Position::new(22, 5), GameOfLife::Alive);
        ..set(Position::new(23, 2), GameOfLife::Alive);
        ..set(Position::new(23, 6), GameOfLife::Alive);
        ..set(Position::new(25, 1), GameOfLife::Alive);
        ..set(Position::new(25, 2), GameOfLife::Alive);
        ..set(Position::new(25, 6), GameOfLife::Alive);
        ..set(Position::new(25, 7), GameOfLife::Alive);
        ..set(Position::new(35, 3), GameOfLife::Alive);
        ..set(Position::new(35, 4), GameOfLife::Alive);
        ..set(Position::new(36, 3), GameOfLife::Alive);
        ..set(Position::new(36, 4), GameOfLife::Alive);
    );
    grid
}

pub fn r_pentomino() -> Grid<GameOfLife> {
    let mut grid = Grid::new(Dimensions::new(201, 201));
    grid = cascade!(
        grid;
        ..set(Position::new(100, 99), GameOfLife::Alive);
        ..set(Position::new(101, 99), GameOfLife::Alive);
        ..set(Position::new(99, 100), GameOfLife::Alive);
        ..set(Position::new(100, 100), GameOfLife::Alive);
        ..set(Position::new(100, 101), GameOfLife::Alive);
    );
    grid
}
