import type { Env, ErrorContext } from "../types";

export class ErrorTracker {
  constructor(private env: Env) {}

  async track(error: Error | unknown, context: ErrorContext): Promise<void> {
    const errorInfo = {
      message: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined,
      context,
      timestamp: new Date().toISOString(),
    };

    // Log to console
    console.error("Error tracked:", errorInfo);

    // Send to Analytics Engine if available
    if (this.env.ANALYTICS) {
      try {
        this.env.ANALYTICS.writeDataPoint({
          blobs: [errorInfo.message, errorInfo.stack || "", JSON.stringify(errorInfo.context)],
          doubles: [Date.now()],
          indexes: ["error", context.action || "unknown"],
        });
      } catch (analyticsError) {
        console.error("Failed to write to Analytics Engine:", analyticsError);
      }
    }

    // Send to external webhook if critical
    if (this.isCritical(error) && this.env.ERROR_WEBHOOK_URL) {
      try {
        await fetch(this.env.ERROR_WEBHOOK_URL, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(errorInfo),
        });
      } catch (webhookError) {
        console.error("Failed to send error webhook:", webhookError);
      }
    }
  }

  private isCritical(error: unknown): boolean {
    if (!(error instanceof Error)) return false;

    const criticalPatterns = ["OutOfMemory", "SecurityError", "DurableObjectError", "NetworkError"];

    return criticalPatterns.some(
      (pattern) => error.message.includes(pattern) || error.name.includes(pattern),
    );
  }
}
