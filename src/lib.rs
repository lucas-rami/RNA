// CELL
pub mod advanced_channels;
pub mod automaton;
pub mod commands;
pub mod simulator;
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
    crate::automaton::game_of_life::GameOfLife;
    "uint cnt_alive = neighbor(Neighbor2D(0, -1)) + neighbor(Neighbor2D(1, -1))\
    + neighbor(Neighbor2D(1, 0)) + neighbor(Neighbor2D(1, 1)) + neighbor(Neighbor2D(0, 1)) \
    + neighbor(Neighbor2D(-1, 1)) + neighbor(Neighbor2D(-1, 0)) + neighbor(Neighbor2D(-1, -1));\
    new_state = uint((state == 0 && cnt_alive == 3) || (state == 1 && (cnt_alive == 2 || cnt_alive == 3)));";
    (crate::universe::grid2d::static_grid2d::StaticGrid2D<crate::automaton::game_of_life::GameOfLife>,
        "shaders/static_2d_grid.comp", gol_static_2d_gird)
}

#[cfg(test)]
mod tests {

    use crate::{
        automaton::game_of_life,
        simulator::{AsyncSimulator, Simulator, SyncSimulator},
    };

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

    #[test]
    fn sync_cpu() {
        let penta_decathlon = game_of_life::penta_decathlon();

        // Run automaton for 14x15=210 generation (14 times the penta-decathlon's period)
        let mut simulator = AsyncSimulator::cpu_backend(penta_decathlon, 10);
        simulator.run(210);

        // Check that the penta-decathlon was updated correctly: each intermediate generation between new periods should be
        // different from the original grid, and the final grid should be identical to the original
        for i in 0..14 {
            let intermediate = simulator.get_generation(i * 16 + 1).unwrap();
            assert!(!game_of_life::is_penta_decathlon(&intermediate));
        }
        let penta_decathlon = simulator.get_generation(210).unwrap();
        assert!(game_of_life::is_penta_decathlon(&penta_decathlon));
    }

    #[test]
    fn async_cpu() {
        let penta_decathlon = game_of_life::penta_decathlon();

        // Run automaton for 14x15=210 generation (14 times the penta-decathlon's period)
        let mut simulator = AsyncSimulator::cpu_backend(penta_decathlon, 10);
        simulator.run(210);

        // Check that the penta-decathlon was updated correctly: each intermediate generation between new periods should be
        // different from the original grid, and the final grid should be identical to the original
        for i in 0..14 {
            let intermediate = simulator.get_generation(i * 16 + 1).unwrap();
            assert!(!game_of_life::is_penta_decathlon(&intermediate));
        }
        let penta_decathlon = simulator.get_generation(210).unwrap();
        assert!(game_of_life::is_penta_decathlon(&penta_decathlon));
    }
}
