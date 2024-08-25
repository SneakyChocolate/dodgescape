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

function send(msg, callback) {
  // fetch("http://100.113.41.134:7878", {
  fetch("http://192.168.178.66:7878/", {
    method: "POST",
    body: msg,
    headers: {
        "Content-type": "application/json"
    }
  })
  .then((r) => r.text())
  .then((r) => callback(r))
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
  let middle = [canvas.width / 2, canvas.height / 2];
  // clear the canvas
  rect(0, 0, canvas.width, canvas.height, "rgb(0,0,0)");
  let shapes = seperate(data, ",");
  // console.log(shapes);
  for (let i = 0; i < shapes.length; i ++) {
    let data = seperate(shapes[i], ",");
    let color = data[0].substring(1,data[0].length - 1);
    let shape = data[1];
    let pos = data[2].substring(2,data[2].length - 1).split(", ");
    let x = Number(pos[0]) * f;
    let y = Number(pos[1]) * f;
    
    if (shape.includes("Circle")) {
      let radius = Number(getattribute(shape, "radius")) * f;
      circle(x + middle[0], y + middle[1], radius, color);
    }
    else if (shape.includes("Rectangle")) {
      let width = getattribute(shape, "width") * f;
      let height = getattribute(shape, "height") * f;
      rect(x + middle[0], y + middle[1], width, height, color);
    }
    else if (shape.includes("Line")) {
      let x2 = Number(getattribute(shape, "x")) * f;
      let y2 = Number(getattribute(shape, "y")) * f;
      let width = Number(getattribute(shape, "width")) * f;
      line([x + middle[0], y + middle[1]], [x2 + middle[0], y2 + middle[1]], width, color);
    }
    else if (shape.includes("Text")) {
      let content = getattribute(shape, "content").substring(1,getattribute(shape, "content").length - 1);
      let size = getattribute(shape, "size") * f;
      // console.log(content, size);
      ctx.fillStyle = color;
      ctx.font = size + "px Arial";
      ctx.fillText(content,x + middle[0], y + middle[1]);
    }
    else if (shape.includes("Poly")) {
      let corners = shape.substring(shape.indexOf("corners: "));
      corners = seperate(corners, ",");
      ctx.fillStyle = color;
      ctx.beginPath();
      for (let i = 0; i < corners.length; i ++) {
        let c = corners[i].trim();
        let xy = c.substring(1, c.length - 1).split(", ");
        let x = Number(xy[0]) * f;
        let y = Number(xy[1]) * f;
        if (i == 0) {
          ctx.moveTo(x + middle[0], y + middle[1]);
        }
        else {
          ctx.lineTo(x + middle[0], y + middle[1]);
        }
      }
      ctx.closePath();
      ctx.fill();
    }
  }
}

login_button.onclick = function(e) {
  let msg = JSON.stringify({mode: "login", username: username.value, x: mouse_x, y: mouse_y, keys_down: keys_down, wheel: wheel});
  // let msg = `let mode = login; let username: String = ${username.value}; let x: i32 = ${mouse_x}; let y: i32 = ${mouse_y}; let keys_down = ${keys_down.join(",")};`;
  send(msg, (r) => {
    // here comes what happens after login
    // output.innerHTML = "output: " + r
    document.body.innerHTML = "";
    canvas = document.createElement("canvas");
    ctx = canvas.getContext("2d");
    document.body.append(canvas);
    canvas.width = window.innerWidth - 3;
    canvas.height = window.innerHeight - 3;
    document.body.style.margin = "0";
    f = canvas.width / 1920;

    // starting canvas action
    renderLoop = setInterval(function() {
      let msg = JSON.stringify({mode: "game", username: username.value, x: mouse_x, y: mouse_y, keys_down: keys_down, wheel: wheel});
      // let msg = `let mode = game; let username: String = ${username.value}; let x: i32 = ${mouse_x}; let y: i32 = ${mouse_y}; let keys_down = ${keys_down.join(",")}; let wheel = ${wheel};`;
      wheel = 0;
      try {
        send(msg, render);
      }
      catch (e) {
        clearInterval(renderLoop);
      }
    }, 30);
  });
}

window.onunload = function () {
  let msg = JSON.stringify({mode: "logout", username: username.value, x: mouse_x, y: mouse_y, keys_down: keys_down, wheel: wheel});
  // let msg = `let mode = logout; let username: String = ${username.value};`;
  send(msg, (r) => {
    console.log(r);
  });
}
