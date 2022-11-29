use simulation::graph::Graph;
use graphics::SimulationScreen;

mod simulation;
mod graphics;

fn main() {    
    let simulation = SimulationScreen::new(1920, 1080);
    simulation.run();
}
