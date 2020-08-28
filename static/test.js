var webSocket = new WebSocket("ws://localhost:8080/websocket");
webSocket.onopen = () => {
    console.log("OPEN");
};
webSocket.onmessage = (ev) => {
    console.log("Receive message: ", ev.data);
};
/*setInterval(() => {
    webSocket.send("Testtext");
}, 3000);*/

setTimeout(() => {
    document.getElementById('btn').addEventListener('click', () => {
        webSocket.send('message '+Math.random());
    });
}, 10);
