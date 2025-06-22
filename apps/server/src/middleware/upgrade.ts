import { createMiddleware } from "hono/factory";

/**
 * Middleware to validate WebSocket upgrade requests
 */
export const upgradeWebSocket = () =>
  createMiddleware(async (c, next) => {
    const upgradeHeader = c.req.header("Upgrade");

    if (upgradeHeader !== "websocket") {
      return c.text("Expected WebSocket upgrade request", 426);
    }

    const connectionHeader = c.req.header("Connection");
    if (!connectionHeader?.toLowerCase().includes("upgrade")) {
      return c.text("Invalid Connection header", 400);
    }

    const wsVersion = c.req.header("Sec-WebSocket-Version");
    if (wsVersion !== "13") {
      return c.text("Unsupported WebSocket version", 400);
    }

    const wsKey = c.req.header("Sec-WebSocket-Key");
    if (!wsKey) {
      return c.text("Missing WebSocket key", 400);
    }

    await next();
  });
