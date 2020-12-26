// CELL
pub mod advanced_channels;
pub mod automaton;
pub mod commands;
pub mod game_of_life;
pub mod universe;

macro_rules! compile_automaton_shaders {
    ($automaton:ty; $update_proc:literal; $cell_type_definition:literal;
        $cell_type:literal; $cell_type_default_value:literal;
        $($universe:ty, $shader_path:literal $mod_name:ident),+) => {

            use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;

            impl crate::automaton::GPUCell for $automaton {}

            $(
                impl crate::universe::UniverseAutomatonShader<$automaton> for $universe {
                    fn shader_info(device: &::std::sync::Arc<vulkano::device::Device>) -> crate::universe::ShaderInfo {
                        let shader = $mod_name::Shader::load(device.clone()).unwrap();
                        let pipeline = vulknao::pipeline::ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap();
                        let layout = pipeline.layout().descriptor_set_layout(0).unwrap().clone();
                        crate::universe::ShaderInfo {
                            layout,
                            pipeline: std::sync::Arc::new(Box::new(pipeline)),
                        }
                    }
                }

                mod $mod_name {
                    vulkano_shaders::shader! {
                        ty: "compute",
                        path: $shader_path,
                        define: [("_UPDATE_PROC_", $update_proc),
                                 ("_CELL_TYPE_DEFINITION_", $cell_type_definition),
                                 ("_CELL_TYPE_", $cell_type),
                                 ("_CELL_TYPE_DEFAULT_VALUE_", $cell_type_default_value)]
                    }
                }
            )+
    };
    ($automaton:ty; $update_proc:literal; $(($universe:ty, $shader_path:literal, $mod_name:ident)),+) => {

            use vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract;

            impl crate::automaton::GPUCell for $automaton {}

            $(
                impl crate::universe::UniverseAutomatonShader<$automaton> for $universe {
                    fn shader_info(device: &::std::sync::Arc<vulkano::device::Device>) -> crate::universe::ShaderInfo {
                        let shader = $mod_name::Shader::load(device.clone()).unwrap();
                        let pipeline = vulkano::pipeline::ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).unwrap();
                        let layout = pipeline.layout().descriptor_set_layout(0).unwrap().clone();
                        crate::universe::ShaderInfo {
                            layout,
                            pipeline: std::sync::Arc::new(Box::new(pipeline)),
                        }
                    }
                }

                mod $mod_name {
                    vulkano_shaders::shader! {
                        ty: "compute",
                        path: $shader_path,
                        define: [("_UPDATE_PROC_", $update_proc)]
                    }
                }
            )+
    };
}

compile_automaton_shaders! {
    game_of_life::GameOfLife;
    "uint cnt_alive = neighbor(Neighbor2D(0, -1)) + neighbor(Neighbor2D(1, -1))\
    + neighbor(Neighbor2D(1, 0)) + neighbor(Neighbor2D(1, 1)) + neighbor(Neighbor2D(0, 1)) \
    + neighbor(Neighbor2D(-1, 1)) + neighbor(Neighbor2D(-1, 0)) + neighbor(Neighbor2D(-1, -1));\
    new_state = uint((state == 0 && cnt_alive == 3) || (state == 1 && (cnt_alive == 2 || cnt_alive == 3)));";
    (crate::universe::grid2d::static_2d_grid::Static2DGrid<game_of_life::GameOfLife>,
        "shaders/static_2d_grid.comp", gol_static_2d_gird)
}

#[cfg(test)]
mod tests {

    use crate::game_of_life;
    use crate::universe::simulator::{AsyncSimulator, SyncSimulator};
    use crate::universe::Simulator;

    #[test]
    fn simple_sync_cpu() {
        // Creates a simple Game of Life's blinker
        let blinker = game_of_life::blinker();

        // Run automaton for 2 generation (the blinker's period)
        let mut simulator = SyncSimulator::cpu_backend(blinker, 10);
        simulator.run(2);

        // Check that the blinker flipped correctly
        let updated_blinker = simulator.get_generation(1).unwrap();
        assert!(game_of_life::is_blinker(&updated_blinker, true));

        // Check that the blinker flipped back to its original shape
        let updated_blinker = simulator.get_generation(2).unwrap();
        assert!(game_of_life::is_blinker(&updated_blinker, false));
    }

    #[test]
    fn simple_async_cpu() {
        // Creates a simple Game of Life's blinker
        let blinker = game_of_life::blinker();

        // Run automaton for 2 generation (the blinker's period)
        let mut simulator = AsyncSimulator::cpu_backend(blinker, 10);
        simulator.run(2);

        // Check that the blinker flipped correctly
        let updated_blinker = simulator.get_generation(1).unwrap();
        assert!(game_of_life::is_blinker(&updated_blinker, true));

        // Check that the blinker flipped back to its original shape
        let updated_blinker = simulator.get_generation(2).unwrap();
        assert!(game_of_life::is_blinker(&updated_blinker, false));
    }
}
