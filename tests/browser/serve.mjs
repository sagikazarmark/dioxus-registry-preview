// The dev server the playwright config's `webServer` block starts.
//
// It serves the preview `dx` already built, rather than building it, because
// the suite runs inside a pinned container that has no rust toolchain in it.
// The build happens outside and only the serving happens here. A dependency
// would do the same job, but the whole of it is forty lines and this way the
// container installs nothing but playwright.

import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, join, resolve } from "node:path";

// Where `dx build --package docs-registry-fixture --bin fixture-preview
// --features web --platform web --release` leaves its output.
const dist = resolve(process.env.DIST ?? "../../target/dx/fixture-preview/release/web/public");
const port = Number(process.env.PORT ?? 8080);

const types = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".ico": "image/x-icon",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".wasm": "application/wasm",
};

async function isFile(path) {
  try {
    return (await stat(path)).isFile();
  } catch {
    return false;
  }
}

const server = createServer(async (request, response) => {
  const { pathname } = new URL(request.url, `http://127.0.0.1:${port}`);

  // Every address in the preview is the same document. The component and the
  // theme are query parameters, so anything that is not a file on disk falls
  // back to the document rather than 404ing.
  let file = resolve(join(dist, decodeURIComponent(pathname)));
  if (!file.startsWith(dist) || !(await isFile(file))) {
    file = join(dist, "index.html");
  }

  if (!(await isFile(file))) {
    response.writeHead(500, { "content-type": "text/plain; charset=utf-8" });
    response.end(`no fixture build at ${dist}: run dx build --package docs-registry-fixture --bin fixture-preview --features web --platform web --release\n`);
    return;
  }

  response.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" });
  createReadStream(file).pipe(response);
});

server.listen(port, "0.0.0.0", () => {
  console.log(`serving ${dist} on http://127.0.0.1:${port}`);
});
