export interface Env {
  // Environment variables
  ALLOWED_ORIGINS?: string;
  MAX_CONNECTIONS_PER_ROOM?: string;
  MAX_MESSAGE_SIZE?: string;
  ERROR_WEBHOOK_URL?: string;

  // Durable Object bindings
  INKRYPT_SYNC_DO: any;

  // Analytics Engine binding (optional)
  ANALYTICS?: any;
}

export interface WebSocketMetadata {
  roomId: string;
  connectedAt: number;
  clientId?: string;
}

export interface ErrorContext {
  roomId?: string;
  clientId?: string;
  action?: string;
  [key: string]: any;
}
