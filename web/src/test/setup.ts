import "@testing-library/jest-dom";
import { afterAll, afterEach, beforeAll } from "vitest";

import { server } from "./mocks/server";

// Start MSW before all tests; reset handlers and close after
beforeAll(() => server.listen({ onUnhandledRequest: "warn" }));
afterEach(() => {
    server.resetHandlers();
    // Clear localStorage so Zustand persist doesn't bleed state between tests
    localStorage.clear();
});
afterAll(() => server.close());
