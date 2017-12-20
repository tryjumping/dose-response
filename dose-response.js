var width = 63;
var height = 43;
var squareSize = 16;
var c = document.createElement('canvas');

c.width = width*squareSize;
c.height = height*squareSize;
document.body.append(c);

var ctx = c.getContext('2d');
ctx.textAlign = "center";
ctx.font = '16px arial';

var wasm_instance;
var gamestate_ptr;
var keyqueue = [];

const keymap = {
  ArrowUp: 0,
  ArrowDown: 1,
  ArrowLeft: 2,
  ArrowRight: 3
};


fetch('target/wasm32-unknown-unknown/release/dose-response.wasm')
  .then(response => response.arrayBuffer())

  .then(bytes => WebAssembly.instantiate(bytes, {
    env: {
      draw: function(ptr, len) {
        console.log("Called draw with ptr:", ptr, "len:", len);
        if(len % 6 != 0) {
          throw new Error("The drawcalls vector must have a multiple of 6 elements!");
        }

        memory = new Uint8Array(wasm_instance.exports.memory.buffer, ptr, len);
        console.log("memory:", memory);

        for(let i = 0; i < len; i += 6) {
          let x = memory[i + 0];
          let y = memory[i + 1];
          let glyph = String.fromCharCode(memory[i + 2]);
          let r = memory[i + 3];
          let g = memory[i + 4];
          let b = memory[i + 5];

          ctx.fillStyle = `rgb(${r},${g},${b})`;
          ctx.clearRect(x * squareSize, y * squareSize, squareSize, squareSize);
          ctx.fillText(glyph, x*squareSize + squareSize / 2, y*squareSize + squareSize / 2);
        }

      }
    }
  }))

  .then(results => {

    document.addEventListener('keydown', (event) => {
      console.log(event);
      keyqueue.push(event);
    });

    console.log("The game has finished.");
    console.log(results);
    console.log(results.module);
    wasm_instance = results.instance;
    gamestate_ptr = results.instance.exports.initialise();
    console.log("The game is initialised.");
    console.log("Gamestate pointer:", gamestate_ptr);


    results.instance.exports.update(gamestate_ptr);

    function update() {
      //window.requestAnimationFrame(update);
      //window.setTimeout(update, 100);
      //console.log("calling update");
      for(let key of keyqueue) {
        var key_code = -1;
        if(key.key in keymap) {
          key_code = keymap[key.key];
        }
        wasm_instance.exports.key_pressed(
          gamestate_ptr,
          key_code,
          key.ctrlKey, key.altKey, key.shiftKey, key.location);
      }
      keyqueue = [];

      results.instance.exports.update(gamestate_ptr);
      //console.log("called update");
    }
    update();
    window.requestAnimationFrame(update);

  });
