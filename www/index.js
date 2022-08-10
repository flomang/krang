import * as sim from "lib-simulation-wasm";

CanvasRenderingContext2D.prototype.drawTriangle = function (x, y, size) {
    this.beginPath();
    this.moveTo(x, y);
    this.lineTo(x + size, y + size);
    this.lineTo(x - size, y + size);
    this.lineTo(x, y);

    this.fillStyle = 'rgb(0, 0, 0)';
    this.fill();
};

const simulation = new sim.Simulation();
const world = simulation.world();

const viewport = document.getElementById('viewport');
const viewportWidth = viewport.width;
const viewportHeight = viewport.height;
const viewportScale = window.devicePixelRatio || 1;

viewport.width = viewportWidth * viewportScale;
viewport.height = viewportHeight * viewportScale;

viewport.style.width = viewportWidth + 'px';
viewport.style.height = viewportHeight + 'px';

// CanvasRenderingContext2D
const ctxt = viewport.getContext('2d');

// Automatically scales all operations by `viewportScale` - otherwise
// we'd have to `* viewportScale` everything by hand
ctxt.scale(viewportScale, viewportScale);

// Rest of the code follows without any changes
ctxt.fillStyle = 'rgb(0, 0, 0)';

for (const animal of simulation.world().animals) {
    ctxt.drawTriangle(
        animal.x * viewportWidth,
        animal.y * viewportHeight,
        0.01 * viewportWidth,
    );
}