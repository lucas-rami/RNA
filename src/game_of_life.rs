// Standard library
use std::collections::HashMap;
use std::sync::Arc;

// External libraries
use cascade::cascade;
use crossterm::style::{style, Attribute, Color, StyledContent};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::{DescriptorSetsCollection, UnsafeDescriptorSetLayout};
use vulkano::descriptor::pipeline_layout::{PipelineLayout, PipelineLayoutAbstract};
use vulkano::device::Device;
use vulkano::pipeline::ComputePipeline;

// CELL
use crate::simulator::gpu_simulator::GPUCompute;
use crate::simulator::grid::{Grid, GridView, Position, RelCoords};
use crate::simulator::{grid::Dimensions, CPUSimulator, CellularAutomaton};
use crate::terminal_ui::TerminalAutomaton;

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash)]
pub enum GOLStates {
    Dead,
    Alive,
}

pub struct GameOfLife {
    name: &'static str,
    style_map: HashMap<GOLStates, StyledContent<char>>,
    vk: Option<VKResources>,
}

struct VKResources {
    pipeline: Arc<ComputePipeline<PipelineLayout<shader::Layout>>>,
    layout: Arc<UnsafeDescriptorSetLayout>,
}

impl GameOfLife {
    pub fn new() -> Self {
        let mut style_map = HashMap::new();
        style_map.insert(GOLStates::Dead, style('·').with(Color::Grey));
        style_map.insert(
            GOLStates::Alive,
            style('#').with(Color::Green).attribute(Attribute::Bold),
        );

        Self {
            name: "Conway's Game of Life",
            style_map,
            vk: None,
        }
    }
}

impl CellularAutomaton<GOLStates> for GameOfLife {
    fn all_states(&self) -> Vec<GOLStates> {
        vec![GOLStates::Dead, GOLStates::Alive]
    }

    fn update_cpu<'a>(&self, grid: &GridView<'a, GOLStates>) -> GOLStates {
        // Count the number of alive cells around us
        let neighbors = vec![
            RelCoords::new(-1, -1),
            RelCoords::new(-1, 0),
            RelCoords::new(-1, 1),
            RelCoords::new(0, 1),
            RelCoords::new(1, 1),
            RelCoords::new(1, 0),
            RelCoords::new(1, -1),
            RelCoords::new(0, -1),
        ];
        let nb_alive_neighbors = grid.get_multiple(neighbors).iter().fold(0, |cnt, cell| {
            if let GOLStates::Alive = cell {
                cnt + 1
            } else {
                cnt
            }
        });

        // Apply the evolution rule
        match grid.state() {
            GOLStates::Dead => {
                if nb_alive_neighbors == 3 {
                    GOLStates::Alive
                } else {
                    GOLStates::Dead
                }
            }
            GOLStates::Alive => {
                if nb_alive_neighbors == 2 || nb_alive_neighbors == 3 {
                    GOLStates::Alive
                } else {
                    GOLStates::Dead
                }
            }
        }
    }
    fn default(&self) -> GOLStates {
        GOLStates::Dead
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl TerminalAutomaton<GOLStates> for GameOfLife {
    fn style(&self, state: &GOLStates) -> &StyledContent<char> {
        &self.style_map.get(state).unwrap()
    }
}

impl GPUCompute<GOLStates> for GameOfLife {
    fn id_from_state(&self, state: &GOLStates) -> u32 {
        match state {
            GOLStates::Dead => 0,
            GOLStates::Alive => 1,
        }
    }

    fn state_from_id(&self, id: u32) -> GOLStates {
        match id {
            0 => GOLStates::Dead,
            1 => GOLStates::Alive,
            _ => panic!("Dummy dum dum"),
        }
    }

    fn bind_device(&mut self, device: &Arc<Device>) -> () {
        let shader = shader::Shader::load(device.clone()).unwrap();
        let pipeline = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap(),
        );
        let layout = pipeline.layout().descriptor_set_layout(0).unwrap().clone();
        self.vk = Some(VKResources { pipeline, layout });
    }

    fn gpu_layout(&self) -> &Arc<UnsafeDescriptorSetLayout> {
        let vk = self
            .vk
            .as_ref()
            .expect("Automaton hasn't been binded to Vulkan device.");
        &vk.layout
    }

    fn gpu_dispatch<U>(
        &self,
        cmd_buffer: AutoCommandBufferBuilder<U>,
        sets: impl DescriptorSetsCollection,
    ) -> AutoCommandBufferBuilder<U> {
        let vk = self
            .vk
            .as_ref()
            .expect("Automaton hasn't been binded to Vulkan device.");
        cmd_buffer
            .dispatch([1, 1, 1], vk.pipeline.clone(), sets, ())
            .unwrap()
    }
}

pub fn conway_canon() -> CPUSimulator<GOLStates, GameOfLife> {
    let gol = GameOfLife::new();
    let mut grid = Grid::new(Dimensions::new(100, 200), &gol.default());
    grid = cascade!(
        grid;
        ..set(&Position::new(1, 5), GOLStates::Alive);
        ..set(&Position::new(1, 6), GOLStates::Alive);
        ..set(&Position::new(2, 5), GOLStates::Alive);
        ..set(&Position::new(2, 6), GOLStates::Alive);
        ..set(&Position::new(11, 5), GOLStates::Alive);
        ..set(&Position::new(11, 6), GOLStates::Alive);
        ..set(&Position::new(11, 7), GOLStates::Alive);
        ..set(&Position::new(12, 4), GOLStates::Alive);
        ..set(&Position::new(12, 8), GOLStates::Alive);
        ..set(&Position::new(13, 3), GOLStates::Alive);
        ..set(&Position::new(13, 9), GOLStates::Alive);
        ..set(&Position::new(14, 3), GOLStates::Alive);
        ..set(&Position::new(14, 9), GOLStates::Alive);
        ..set(&Position::new(15, 6), GOLStates::Alive);
        ..set(&Position::new(16, 4), GOLStates::Alive);
        ..set(&Position::new(16, 8), GOLStates::Alive);
        ..set(&Position::new(17, 5), GOLStates::Alive);
        ..set(&Position::new(17, 6), GOLStates::Alive);
        ..set(&Position::new(17, 7), GOLStates::Alive);
        ..set(&Position::new(18, 6), GOLStates::Alive);
        ..set(&Position::new(21, 3), GOLStates::Alive);
        ..set(&Position::new(21, 4), GOLStates::Alive);
        ..set(&Position::new(21, 5), GOLStates::Alive);
        ..set(&Position::new(22, 3), GOLStates::Alive);
        ..set(&Position::new(22, 4), GOLStates::Alive);
        ..set(&Position::new(22, 5), GOLStates::Alive);
        ..set(&Position::new(23, 2), GOLStates::Alive);
        ..set(&Position::new(23, 6), GOLStates::Alive);
        ..set(&Position::new(25, 1), GOLStates::Alive);
        ..set(&Position::new(25, 2), GOLStates::Alive);
        ..set(&Position::new(25, 6), GOLStates::Alive);
        ..set(&Position::new(25, 7), GOLStates::Alive);
        ..set(&Position::new(35, 3), GOLStates::Alive);
        ..set(&Position::new(35, 4), GOLStates::Alive);
        ..set(&Position::new(36, 3), GOLStates::Alive);
        ..set(&Position::new(36, 4), GOLStates::Alive);
    );
    CPUSimulator::new("Conway Cannon", gol, &grid)
}

mod shader {
    vulkano_shaders::shader! {
        ty: "compute",
        src: "
             #version 450
             layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
             layout(set = 0, binding = 0) buffer Data {
                 uint data[];
             } data;

             void main() {
                 uint idx = gl_GlobalInvocationID.x;
                 data.data[idx] *= 12;
             }
         "
    }
}
