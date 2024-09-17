let login_button = document.getElementById("login");
let username = document.getElementById("nameinput");
let canvas;
let ctx;
let keys_down = [];
let wheel = 0;

document.onwheel = function(e) {
  wheel = e.deltaY;
}

document.onkeydown = function(e) {
 	if (keys_down.includes(e.code)) {
		return;
	}
	keys_down.push(e.code);
}

document.onkeyup = function(e) {
	let index = keys_down.indexOf(e.code);
	if (index >= 0) {
		keys_down.splice(index, 1);
	}
}

function seperate(string, seperator) {
	let elements = [];
	let last = -1;
	let layer = 0;
	for (let i = 0; i < string.length; i ++) {
		let char = string[i];
		if (char == seperator && layer == 1) {
			elements.push(string.substring(last + 1, i));
			last = i;
		}
		else if (char == "[" || char == "(" || char == "{") {
			if (layer == 0) {
				last = i;
			}
			layer ++;
		}
		else if (char == "]" || char == ")" || char == "}") {
			if (layer == 1) {
				elements.push(string.substring(last + 1, i));
			}
			layer --;
		}
	}
	return elements;
}

let f = 1;
function circle(x, y, radius, color) {
    ctx.beginPath();
    ctx.arc(
        x,
        y,
        radius,
        0, 2 * Math.PI);
    ctx.fillStyle = color;
    ctx.fill();
    ctx.closePath();
}

function rect(x, y, width, height, color) {
    ctx.beginPath();
    ctx.rect(x, y, width, height);
    ctx.fillStyle = color;
    ctx.fill();
    ctx.closePath();
}

function line(start, end, width, color) {
    ctx.lineWidth = width;
    ctx.strokeStyle = color;

    ctx.beginPath();
    ctx.moveTo( start[0], start[1] );
    ctx.lineTo( end[0], end[1] );
    ctx.stroke();
    ctx.closePath();
}

let renderLoop;
let mouse_x = 0;
let mouse_y = 0;

document.body.onmousemove = function(e) {
  try {
    mouse_x = e.clientX - canvas.width / 2;
    mouse_y = e.clientY - canvas.height / 2;
  }
  catch (error) {
    
  }
}

function findClosest(string, words, offset) {
  let min = -1;
  for (let i = 0; i < words.length; i ++) {
    let word = words[i];
    let index = string.indexOf(word, offset);
    if ((index < min || min == -1) && index >= 0) {
      min = index;
    }
  }
  return min;
}

function getattribute(string, attribute) {
  let signature = attribute + ": ";
  let index = string.indexOf(signature) + signature.length;
  let value = string.substring(index, findClosest(string, [" ", ","], index));
  return value;
}

function render(data) {
  // clear the canvas
  rect(0, 0, canvas.width, canvas.height, "rgb(0,0,0)");
  let middle = [canvas.width / 2, canvas.height / 2];
  let objects = "";
  try {
    objects = JSON.parse(data).objects; 
  }
  catch (e) {
    console.log("panic");
    return;
  }
  for (o in objects) {
    let object = objects[o];
    if (object == null) continue;
    let x = (object.position.x - object.camera.x + object.draw_pack.offset[0]) * object.zoom * f + middle[0];
    let y = (object.position.y - object.camera.y + object.draw_pack.offset[1]) * object.zoom * f + middle[1];

    for (s in object.draw_pack.shape) {
      let shape = object.draw_pack.shape[s];
      if (s == "Circle") {
        circle(x, y, shape.radius * object.zoom * f, object.draw_pack.color);
      }
      else if (s == "Rectangle") {
        rect(x, y, shape.width * object.zoom * f, shape.height * object.zoom * f, object.draw_pack.color);
      }
      else if (s == "Line") {
        let x2 = (shape.x - object.camera.x) * object.zoom * f + middle[0];
        let y2 = (shape.y - object.camera.y) * object.zoom * f + middle[1];
        line([x, y], [x2, y2], shape.width * object.zoom * f, object.draw_pack.color);
      }
      else if (s == "Text") {
        ctx.fillStyle = object.draw_pack.color;
        ctx.font = shape.size * object.zoom * f + "px Arial";
        ctx.fillText(shape.content, x, y);
      }
      else if (s == "Poly") {
        let corners = shape.corners;
        ctx.fillStyle = object.draw_pack.color;
        ctx.beginPath();
        for (c in corners) {
          let corner = corners[c];
          let x = (corner[0] - object.camera.x) * object.zoom * f + middle[0];
          let y = (corner[1] - object.camera.y) * object.zoom * f + middle[1];
          if (c == 0) {
            ctx.moveTo(x, y);
          }
          else {
            ctx.lineTo(x, y);
          }
        }
        ctx.closePath();
        ctx.fill();
      }
    }
  }
  return;
}

login_button.onclick = function(_e) {
  let ws = new WebSocket("ws://127.0.0.1:7878/");
  ws.onopen = function() {
    // here comes what happens after login
    let loginmsg = JSON.stringify({mode: "login", username: username.value, x: mouse_x, y: mouse_y, keys_down: keys_down, wheel: wheel});
    ws.send(loginmsg);
    document.body.innerHTML = "";
    canvas = document.createElement("canvas");
    ctx = canvas.getContext("2d");
    document.body.append(canvas);
    canvas.width = window.innerWidth - 3;
    canvas.height = window.innerHeight - 3;
    document.body.style.margin = "0";
    f = canvas.width / 1920;

    ws.onmessage = function(e) {
      render(e.data);
    };
    ws.onclose = function() {
      clearInterval(renderLoop);
    }

    // starting canvas action
    renderLoop = setInterval(function() {
      let gamemsg = JSON.stringify({mode: "game", username: username.value, x: mouse_x, y: mouse_y, keys_down: keys_down, wheel: wheel});
      // let msg = `let mode = game; let username: String = ${username.value}; let x: i32 = ${mouse_x}; let y: i32 = ${mouse_y}; let keys_down = ${keys_down.join(",")}; let wheel = ${wheel};`;
      wheel = 0;
      try {
        ws.send(gamemsg);
      }
      catch (e) {
        clearInterval(renderLoop);
      }
    }, 30);
  }
}

