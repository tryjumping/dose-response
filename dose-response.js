var width = 80;
var height = 60;
var squareSize = 10;
var c = document.createElement('canvas');



c.width = width*squareSize;
c.height = height*squareSize;
document.body.append(c);

var ctx = c.getContext('2d');
ctx.textAlign = "center";
ctx.font = '12px arial';

var wasm_instance;
var gamestate_ptr;


fetch('target/wasm32-unknown-unknown/release/dose-response.wasm')
  .then(response => response.arrayBuffer())

  .then(bytes => WebAssembly.instantiate(bytes, {
    env: {
      draw: function(ptr, len) {
        console.log("Called draw with ptr:", ptr, "len:", len);

        memory = new Uint8Array(wasm_instance.exports.memory.buffer, ptr, len);
        console.log("memory:", memory);

        for(let n of memory.values()) {
          console.log(n);
        }

      }
    }
  }))

  .then(results => {
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
      //console.log("calling update");
      results.instance.exports.update(gamestate_ptr);
      //console.log("called update");
    }
    update();
    window.requestAnimationFrame(update);

  });
