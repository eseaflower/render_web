import { entry, RenderController } from "render_web";

let canvas = document.getElementById("foo");
let controller = null;
let animationHandle = null;
let count = 0;

function doRender() {
    controller.render();
    count++;
    if (count < 3) {
        animationHandle = requestAnimationFrame(doRender);
    }
}

async function start() {
    controller = await RenderController.new(1);
    console.log("After");
    //controller.swap_image();
    animationHandle = requestAnimationFrame(doRender);
}

start().then(e => console.log("Promise done"));




//entry();


