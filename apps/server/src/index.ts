import { Hono } from "hono";

const app = new Hono();

app.get("/", (c) => {
  return c.text("Hello Inkrypt user, I'm Hono!");
});

export default app;
