// CELL
mod compute;
mod simulator;
pub use simulator::Simulator;

#[cfg(test)]
mod tests {

    // External libraries
    use vulkano::instance::{Instance, InstanceExtensions};

    // CELLs
    use super::Simulator;
    use crate::automaton::{UpdateCPU, CellularAutomaton};
    use crate::game_of_life::*;
    use crate::grid::{Grid, MOORE_NEIGHBORHOOD};

    #[test]
    fn cpu_get_multiple_gens() {
        let grid = gosper_glider_gun();
        let automaton = CellularAutomaton::new("Automaton", &MOORE_NEIGHBORHOOD);
        let sim = Simulator::new_cpu_sim(automaton, &grid);
        get_multiple_gens(sim, &grid, vec![100, 1, 7, 10, 19, 20]);
    }

    #[test]
    fn gpu_get_multiple_gens() {
        let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
        let grid = gosper_glider_gun();
        let automaton = CellularAutomaton::new("Automaton", &MOORE_NEIGHBORHOOD);
        let sim = Simulator::new_gpu_sim(automaton, &grid, instance);
        get_multiple_gens(sim, &grid, vec![100, 1, 7, 10, 19, 20]);
    }

    fn get_multiple_gens<C: UpdateCPU>(
        mut sim: Simulator<C>,
        initial_grid: &Grid<C>,
        gens: Vec<usize>,
    ) {
        let ref_grids: Vec<Grid<C>> = gens
            .iter()
            .map(|gen| compute_gen::<C>(&initial_grid, *gen))
            .collect();

        for (gen, grid) in gens.iter().zip(ref_grids.iter()) {
            assert_eq!(grid, &sim.get_gen(*gen, true).unwrap());
        }
    }

    fn compute_gen<C: UpdateCPU>(base: &Grid<C>, nb_gens: usize) -> Grid<C> {
        let mut grid = base.clone();
        for _i in 0..nb_gens {
            grid = C::update_grid(&grid);
        }
        grid
    }
}
