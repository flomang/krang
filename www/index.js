import * as sim from "lib-simulation-wasm";

const simulation = new sim.Simulation();
const world = simulation.world();
console.log(world);

alert("Who's that dog? " + sim.whos_that_dog() + "!");