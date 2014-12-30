console.log("starting websockting");


var ws = new WebSocket("ws://127.0.0.1:8099/ws", "protocolOne");

ws.onopen = function (evt) {
    console.log("open", evt);

    ws.send("an excellent client message");

    setTimeout(function () {
	ws.send("stay fresh websocket bag");


	// TODO: run this with > 125
	var ab = new ArrayBuffer(125);
	var someinfo = new Uint8Array(ab, 0, 125); 
	
	ws.send(someinfo);
    }, 1000);

}

ws.onclose = function (evt) {
    console.log("close", evt);
};


ws.onmessage = function (evt) {
    console.log("msg", evt, typeof evt.data);

    if (evt.data instanceof Blob) {
	console.log("Thats a blob!");
	var reader = new window.FileReader();
	reader.readAsDataURL(evt.data); 
	reader.onloadend = function() {
            base64data = reader.result;                
            console.log(base64data);
	}
	
    }
};
