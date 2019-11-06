import init from '/dist/index.js';
import { start } from '/dist/index.js';

// window.enableClock = () => {
//     const sendTick = () => {
//         tick(new Date().toLocaleTimeString());
//     };
//     sendTick();

//     setInterval(() => {
//         sendTick();
//     }, 1000);
// };

init('/pkg/package_bg.wasm').then(() => {
    const js_ready = start();
    // window.tick = tick;
    js_ready(true);
});


