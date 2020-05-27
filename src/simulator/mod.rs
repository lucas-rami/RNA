// CELL
mod simulator;
mod compute;
pub mod advanced_channels;
pub use simulator::Simulator;

#[cfg(test)]
mod tests {

    // External libraries
    use vulkano::instance::{Instance, InstanceExtensions};

    // CELLs
    use super::Simulator;
    use crate::automaton::CPUComputableAutomaton;
    use crate::game_of_life::*;
    use crate::grid::Grid;

    #[test]
    fn cpu_get_multiple_gens() {
        let grid = gosper_glider_gun();
        let sim = Simulator::new_cpu_sim("Simulator", GameOfLife::new(), &grid);
        get_multiple_gens(sim, &grid, vec![100, 1, 7, 10, 19, 20]);
    }

    #[test]
    fn gpu_get_multiple_gens() {
        let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
        let grid = gosper_glider_gun();
        let sim = Simulator::new_gpu_sim("Simulator", GameOfLife::new(), &grid, instance);
        get_multiple_gens(sim, &grid, vec![100, 1, 7, 10, 19, 20]);
    }

    fn get_multiple_gens<A: CPUComputableAutomaton>(
        mut sim: Simulator<A>,
        initial_grid: &Grid<A::Cell>,
        gens: Vec<usize>,
    ) {
        let ref_grids: Vec<Grid<A::Cell>> = gens
            .iter()
            .map(|gen| compute_gen::<A>(&initial_grid, *gen))
            .collect();

        for (gen, grid) in gens.iter().zip(ref_grids.iter()) {
            assert_eq!(grid, &sim.get_gen(*gen, true).unwrap());
        }
    }

    fn compute_gen<A: CPUComputableAutomaton>(
        base: &Grid<A::Cell>,
        nb_gens: usize,
    ) -> Grid<A::Cell> {
        let mut grid = base.clone();
        for _i in 0..nb_gens {
            grid = A::update_grid(&grid);
        }
        grid
    }
}
