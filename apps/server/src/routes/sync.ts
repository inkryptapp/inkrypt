import { Hono } from "hono";
import { upgradeWebSocket } from "../middleware/upgrade";
import type { Env } from "../types";

export const syncRoute = new Hono<{ Bindings: Env }>()
  // WebSocket upgrade endpoint
  .get(
    "/:roomId",
    //authenticate(),
    upgradeWebSocket(),
    async (c) => {
      const roomId = c.req.param("roomId");

      if (!roomId) {
        return c.text("Room ID is required", 400);
      }

      // Validate room ID format (alphanumeric and hyphens only)
      if (!/^[a-zA-Z0-9-]+$/.test(roomId)) {
        return c.text("Invalid room ID format", 400);
      }

      try {
        console.log("Environment:", c.env);
        console.log("INKRYPT_SYNC_DO:", c.env?.INKRYPT_SYNC_DO);

        if (!c.env?.INKRYPT_SYNC_DO) {
          console.error("INKRYPT_SYNC_DO binding not found in environment");
          return c.text("Durable Object binding not configured", 500);
        }

        // Get the Durable Object instance for this room
        const id = c.env.INKRYPT_SYNC_DO.idFromName(roomId);
        const stub = c.env.INKRYPT_SYNC_DO.get(id);

        // Forward the request to the Durable Object
        const response = await stub.fetch(c.req.raw);

        return response;
      } catch (error) {
        console.error("Error connecting to room:", error);
        return c.text("Failed to connect to room", 500);
      }
    },
  );
