# Websocket Protocol

True Gear CLI exposes a WebSocket server that allows clients to interact with True Gear devices over a network connection. The WebSocket server listens for incoming connections and processes messages on `ws://127.0.0.1:18233/v1/tact/`.

The WebSocket server accepts JSON-formatted messages that conform to the schema defined in `effect.schema.json`. Clients can send effect commands to control the True Gear device's behavior.

