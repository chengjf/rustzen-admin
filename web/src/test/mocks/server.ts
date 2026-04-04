import { setupServer } from "msw/node";

import { handlers } from "./handlers";

/** MSW server instance used by Vitest (Node environment). */
export const server = setupServer(...handlers);
