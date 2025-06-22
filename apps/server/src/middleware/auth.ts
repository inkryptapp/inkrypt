import { createMiddleware } from "hono/factory";
import type { Env } from "../types";

/**
 * Simple authentication middleware
 * In production, this should validate JWT tokens or session cookies
 */
export const authenticate = () =>
  createMiddleware<{ Bindings: Env }>(async (c, next) => {
    // Get authorization header
    const authHeader = c.req.header("Authorization");

    // For development, allow connections without auth if ALLOWED_ORIGINS is "*"
    if (c.env.ALLOWED_ORIGINS === "*") {
      await next();
      return;
    }

    if (!authHeader) {
      return c.text("Unauthorized", 401);
    }

    // Extract token (Bearer token format)
    const token = authHeader.replace(/^Bearer\s+/i, "");

    if (!token) {
      return c.text("Invalid authorization format", 401);
    }

    try {
      // TODO: Implement actual token validation
      // For now, this is a placeholder that accepts any non-empty token
      // In production, you would:
      // 1. Validate JWT signature
      // 2. Check token expiration
      // 3. Verify user permissions for the requested room

      if (token.length < 10) {
        return c.text("Invalid token", 401);
      }

      // Set user context (would come from token validation)
      c.set("user", {
        id: `user-${crypto.randomUUID()}`,
        token: token,
      });

      await next();
    } catch (error) {
      console.error("Authentication error:", error);
      return c.text("Authentication failed", 401);
    }
  });
