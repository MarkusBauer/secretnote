
function x() {
    var webSocket = new WebSocket("ws://localhost:8080/websocket");
    webSocket.onopen = () => {
        console.log("OPEN");
    };
    webSocket.onmessage = (ev) => {
        console.log("Receive message: ", ev.data);
    };
    setInterval(() => {
        webSocket.send("Testtext");
    }, 3000);
}

x();
