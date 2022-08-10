import * as sim from "lib-simulation-wasm";

function redraw() {
    ctxt.clearRect(0, 0, viewportWidth, viewportHeight);

    simulation.step();

    for (const animal of simulation.world().animals) {
        ctxt.drawTriangle(
            animal.x * viewportWidth,
            animal.y * viewportHeight,
            0.01 * viewportWidth,
            animal.rotation,
        );
    }

    // requestAnimationFrame() schedules code only for the next frame.
    //
    // Because we want for our simulation to continue forever, we've
    // gotta keep re-scheduling our function:
    requestAnimationFrame(redraw);
}

CanvasRenderingContext2D.prototype.drawTriangle =
    function (x, y, size, rotation) {
        this.beginPath();

        this.moveTo(
            x + Math.cos(rotation) * size * 1.5,
            y + Math.sin(rotation) * size * 1.5,
        );

        this.lineTo(
            x + Math.cos(rotation + 2.0 / 3.0 * Math.PI) * size,
            y + Math.sin(rotation + 2.0 / 3.0 * Math.PI) * size,
        );

        this.lineTo(
            x + Math.cos(rotation + 4.0 / 3.0 * Math.PI) * size,
            y + Math.sin(rotation + 4.0 / 3.0 * Math.PI) * size,
        );

        this.lineTo(
            x + Math.cos(rotation) * size * 1.5,
            y + Math.sin(rotation) * size * 1.5,
        );

        this.stroke();
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
redraw();
