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
use crate::simulator::grid::{Grid, GridView, Position, RelCoords};
use crate::simulator::GPUCompute;
use crate::simulator::{grid::Dimensions, CellularAutomaton};
use crate::terminal_ui::TerminalAutomaton;

pub struct GameOfLife {
    name: &'static str,
    style_map: HashMap<States, StyledContent<char>>,
    vk: Option<VKResources>,
}

struct VKResources {
    pipeline: Arc<ComputePipeline<PipelineLayout<shader::Layout>>>,
    layout: Arc<UnsafeDescriptorSetLayout>,
}

impl GameOfLife {
    pub fn new() -> Self {
        let mut style_map = HashMap::new();
        style_map.insert(States::Dead, style('·').with(Color::Grey));
        style_map.insert(
            States::Alive,
            style('#').with(Color::Green).attribute(Attribute::Bold),
        );

        Self {
            name: "Conway's Game of Life",
            style_map,
            vk: None,
        }
    }
}

impl CellularAutomaton<States> for GameOfLife {
    fn all_states(&self) -> Vec<States> {
        vec![States::Dead, States::Alive]
    }

    fn update_cpu<'a>(&self, grid: &GridView<'a, States>) -> States {
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
            if let States::Alive = cell {
                cnt + 1
            } else {
                cnt
            }
        });

        // Apply the evolution rule
        match grid.state() {
            States::Dead => {
                if nb_alive_neighbors == 3 {
                    States::Alive
                } else {
                    States::Dead
                }
            }
            States::Alive => {
                if nb_alive_neighbors == 2 || nb_alive_neighbors == 3 {
                    States::Alive
                } else {
                    States::Dead
                }
            }
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl TerminalAutomaton<States> for GameOfLife {
    fn style(&self, state: &States) -> &StyledContent<char> {
        &self.style_map.get(state).unwrap()
    }
}

impl GPUCompute<States> for GameOfLife {
    fn id_from_state(&self, state: &States) -> u32 {
        match state {
            States::Dead => 0,
            States::Alive => 1,
        }
    }

    fn state_from_id(&self, id: u32) -> States {
        match id {
            0 => States::Dead,
            1 => States::Alive,
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

#[derive(Copy, Clone, Eq, PartialEq, std::hash::Hash)]
pub enum States {
    Dead,
    Alive,
}

impl Default for States {
    fn default() -> Self {
        Self::Dead
    }
}


pub fn conway_canon() -> Grid<States> {
    let mut grid = Grid::new(Dimensions::new(100, 200), States::default());
    grid = cascade!(
        grid;
        ..set(&Position::new(1, 5), States::Alive);
        ..set(&Position::new(1, 6), States::Alive);
        ..set(&Position::new(2, 5), States::Alive);
        ..set(&Position::new(2, 6), States::Alive);
        ..set(&Position::new(11, 5), States::Alive);
        ..set(&Position::new(11, 6), States::Alive);
        ..set(&Position::new(11, 7), States::Alive);
        ..set(&Position::new(12, 4), States::Alive);
        ..set(&Position::new(12, 8), States::Alive);
        ..set(&Position::new(13, 3), States::Alive);
        ..set(&Position::new(13, 9), States::Alive);
        ..set(&Position::new(14, 3), States::Alive);
        ..set(&Position::new(14, 9), States::Alive);
        ..set(&Position::new(15, 6), States::Alive);
        ..set(&Position::new(16, 4), States::Alive);
        ..set(&Position::new(16, 8), States::Alive);
        ..set(&Position::new(17, 5), States::Alive);
        ..set(&Position::new(17, 6), States::Alive);
        ..set(&Position::new(17, 7), States::Alive);
        ..set(&Position::new(18, 6), States::Alive);
        ..set(&Position::new(21, 3), States::Alive);
        ..set(&Position::new(21, 4), States::Alive);
        ..set(&Position::new(21, 5), States::Alive);
        ..set(&Position::new(22, 3), States::Alive);
        ..set(&Position::new(22, 4), States::Alive);
        ..set(&Position::new(22, 5), States::Alive);
        ..set(&Position::new(23, 2), States::Alive);
        ..set(&Position::new(23, 6), States::Alive);
        ..set(&Position::new(25, 1), States::Alive);
        ..set(&Position::new(25, 2), States::Alive);
        ..set(&Position::new(25, 6), States::Alive);
        ..set(&Position::new(25, 7), States::Alive);
        ..set(&Position::new(35, 3), States::Alive);
        ..set(&Position::new(35, 4), States::Alive);
        ..set(&Position::new(36, 3), States::Alive);
        ..set(&Position::new(36, 4), States::Alive);
    );
    grid
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
