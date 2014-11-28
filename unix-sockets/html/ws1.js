console.log("starting websockting");


var ws = new WebSocket("ws://127.0.0.1:8099/ws", "protocolOne");

ws.onopen = function (evt) {
    console.log("open", evt);
}

ws.onclose = function (evt) {
    console.log("close", evt);
};


ws.onmessage = function (evt) {
    console.log("msg", evt);
};
