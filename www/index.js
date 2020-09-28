import { entry, RenderController } from "render_web";

let canvas = document.getElementById("foo");
let old_cursor = null;
let mouseDown = false;
let ctrl = false;

canvas.onmousedown = (evt) => {
    if (evt.buttons === 1) {
        mouseDown = true;
        old_cursor = canvas.style.cursor;
        canvas.style.cursor = 'none';
    }
}
canvas.onmouseup = (evt) => {
    mouseDown = false;
    controller.clear_anchor();
    canvas.style.cursor = old_cursor;
}

canvas.onmousemove = (evt) => {
    if (mouseDown) {
        if (evt.ctrlKey) {
            controller.update_zoom(evt.offsetX, evt.offsetY);
        } else {
            controller.update_position(evt.offsetX, evt.offsetY);
        }
        count = 0;
        if (animationHandle === null) {
            doRender();
        }
    }
}


let controller = null;
let animationHandle = null;
let count = 0;

function doRender() {
    controller.render();
    count++;
    if (count < 3) {
        animationHandle = requestAnimationFrame(doRender);
    } else {
        animationHandle = null;
    }
}

async function start() {
    controller = await RenderController.new(1, canvas.clientWidth, canvas.clientHeight);

    console.log("After");
    //controller.swap_image();
    animationHandle = requestAnimationFrame(doRender);
}

start().then(e => console.log("Promise done"));




//entry();


