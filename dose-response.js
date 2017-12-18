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

var rust_memory;
var buffer_pointer;


fetch('target/wasm32-unknown-unknown/release/dose-response.wasm')
  .then(response => response.arrayBuffer())

  .then(bytes => WebAssembly.instantiate(bytes, {
    env: {
      draw: function(ptr) {
        console.log("Called draw with:", arguments);
        console.log("ptr: ", ptr);
        console.log("rust mem:", rust_memory);


        for(var i = 0; i < 10; i++) {
          console.log("arr[ptr]:", rust_memory[(ptr) + i]);
        }

        console.log("buffer pointer: ", buffer_pointer);
        for(var i = 0; i < 10; i++) {
          console.log("arr[buffer_pointer]:", rust_memory[(buffer_pointer) + i]);
        }

      }
    }
  }))

  .then(results => {
    console.log("The game has finished.");
    console.log(results);
    console.log(results.module);
    rust_memory = new Uint8Array(results.instance.exports.memory.buffer);
    let buffer_ptr = results.instance.exports.initialise();
    rust_memory[buffer_ptr+1] = 22;
    console.log("The game is initialised.");
    console.log("Buffer pointer:", buffer_ptr);
    buffer_pointer = buffer_ptr;
    results.instance.exports.update(buffer_ptr);

    function update() {
      //window.requestAnimationFrame(update);
      //console.log("calling update");
      results.instance.exports.update(buffer_ptr);
      //console.log("called update");
    }
    update();
    window.requestAnimationFrame(update);

  });
