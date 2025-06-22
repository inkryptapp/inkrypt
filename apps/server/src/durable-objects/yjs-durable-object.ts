import type { Env, WebSocketMetadata } from "../types";

export class YjsDurableObject {
  private sessions: Map<WebSocket, WebSocketMetadata>;

  constructor(
    private state: any,
    private env: Env,
  ) {
    this.sessions = new Map();
    console.log("YjsDurableObject instantiated");
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    // Check if this is a WebSocket upgrade request
    const upgradeHeader = request.headers.get("Upgrade");
    if (!upgradeHeader || upgradeHeader !== "websocket") {
      return new Response("Expected WebSocket", { status: 426 });
    }

    // Check if room is at capacity
    const maxConnections = parseInt(this.env.MAX_CONNECTIONS_PER_ROOM || "1000", 10);
    if (this.sessions.size >= maxConnections) {
      return new Response("Room at capacity", {
        status: 503,
        headers: { "Retry-After": "60" },
      });
    }

    // Extract room ID from URL
    const roomId = url.pathname.split("/").pop() || "default";

    // Create WebSocket pair
    const pair = new WebSocketPair();
    const client = pair[0];
    const server = pair[1];

    // Store metadata
    const metadata: WebSocketMetadata = {
      roomId,
      connectedAt: Date.now(),
      clientId: crypto.randomUUID(),
    };

    // Accept the WebSocket connection
    this.state.acceptWebSocket(server, ["yjs-sync"]);

    // Track the session
    this.sessions.set(server, metadata);

    // Return the client WebSocket
    return new Response(null, {
      status: 101,
      webSocket: client,
    } as any);
  }

  async webSocketMessage(ws: WebSocket, message: ArrayBuffer | string) {
    // Validate message size
    const messageSize = message instanceof ArrayBuffer ? message.byteLength : message.length;
    const maxSize = parseInt(this.env.MAX_MESSAGE_SIZE || "1048576", 10);

    if (messageSize > maxSize) {
      ws.close(1009, "Message too large");
      return;
    }

    // Get sender metadata
    const senderMetadata = this.sessions.get(ws);
    if (!senderMetadata) return;

    // Forward message to all other connections in the same room
    let successCount = 0;
    let errorCount = 0;

    for (const [session, metadata] of this.sessions) {
      // Skip sender and different rooms
      if (session === ws || metadata.roomId !== senderMetadata.roomId) {
        continue;
      }

      // Only send to open connections
      if (session.readyState === WebSocket.OPEN) {
        try {
          session.send(message);
          successCount++;
        } catch (error) {
          console.error("Failed to forward message:", error);
          errorCount++;
          this.sessions.delete(session);
        }
      }
    }

    // Simple logging
    console.log(
      `Forwarded message in room ${senderMetadata.roomId}: ${successCount} success, ${errorCount} errors`,
    );
  }

  async webSocketClose(ws: WebSocket, code: number, reason: string, wasClean: boolean) {
    const metadata = this.sessions.get(ws);
    this.sessions.delete(ws);

    // Log disconnection
    if (metadata) {
      console.log(`Client ${metadata.clientId} disconnected from room ${metadata.roomId}`, {
        code,
        reason,
        wasClean,
        duration: Date.now() - metadata.connectedAt,
      });
    }
  }

  async webSocketError(ws: WebSocket, error: unknown) {
    const metadata = this.sessions.get(ws);
    this.sessions.delete(ws);

    console.error("WebSocket error:", error, {
      clientId: metadata?.clientId,
      roomId: metadata?.roomId,
    });
  }
}
