import { Hono } from "hono";
import { cors } from "hono/cors";
import { logger } from "hono/logger";
import { syncRoute } from "./routes/sync";
import type { Env } from "./types";

// Export Durable Objects
export { YjsDurableObject } from "./durable-objects/yjs-durable-object";

// Create Hono app
const app = new Hono<{ Bindings: Env }>();

// Global middleware
app.use("*", logger());
app.use(
  "*",
  cors({
    origin: (origin, c) => {
      const allowedOrigins = (c.env?.ALLOWED_ORIGINS || "*").split(",").map((o) => o.trim());
      if (allowedOrigins.includes("*") || allowedOrigins.includes(origin)) {
        return origin;
      }
      return null;
    },
    credentials: true,
  }),
);

// Routes
app.route("/sync", syncRoute);

// 404 handler
app.notFound((c) => c.text("Not Found", 404));

// Error handler
app.onError((err, c) => {
  console.error("Unhandled error:", err);
  return c.text("Internal Server Error", 500);
});

export default app;
