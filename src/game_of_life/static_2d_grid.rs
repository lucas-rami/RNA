// // Standard library
// use std::sync::Arc;

// // External libraries
// use vulkano::descriptor::pipeline_layout::{PipelineLayout, PipelineLayoutAbstract};
// use vulkano::device::Device;
// use vulkano::pipeline::ComputePipeline;

// // CELL
// use super::GameOfLife;
// use crate::automaton::{GPUCell, ShaderInfo};
// use crate::universe::grid2d::static_grid::Static2DGrid;

// impl GPUCell<Static2DGrid<GameOfLife>> for GameOfLife {
//     type Pipeline = ComputePipeline<PipelineLayout<shader::Layout>>;

//     fn shader_info(device: &Arc<Device>) -> ShaderInfo<Self::Pipeline> {
//         let shader = shader::Shader::load(device.clone()).unwrap();
//         let pipeline =
//             ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap();
//         let layout = pipeline.layout().descriptor_set_layout(0).unwrap().clone();
//         ShaderInfo {
//             layout,
//             pipeline: Box::new(Arc::new(pipeline)),
//         }
//     }
// }

// mod shader {
//     vulkano_shaders::shader! {   
//         ty: "compute",
//         path: "shaders/static_2d_grid.comp",
//     }
// }

// // uint new_state = 0;

// // if (x >= 1 && x < grid_size.height - 1 && y >= 1 && y < grid_size.width - 1) {
// //     uint cnt_alive = 0;
// //     for (uint i = 0 ; i < 8 ; i++) {
// //         ivec2 nbor = neighbors[i];
// //         uint nx = x + nbor.x;
// //         uint ny = y + nbor.y;
// //         if (auto_in.data[nx + ny * grid_size.height] == 1) {
// //            cnt_alive++;
// //         }
// //     }
// //     new_state = uint((current_state == 0 && cnt_alive == 3) || (current_state == 1 && (cnt_alive == 2 || cnt_alive == 3)));
// // }
