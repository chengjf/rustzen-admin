import { defineConfig, mergeConfig } from "vitest/config";

import viteConfig from "./vite.config";

export default mergeConfig(
    viteConfig,
    defineConfig({
        test: {
            environment: "jsdom",
            globals: true,
            setupFiles: ["./src/test/setup.ts"],
            coverage: {
                provider: "v8",
                reporter: ["text", "lcov"],
                include: ["src/**/*.{ts,tsx}"],
                exclude: [
                    "src/routeTree.gen.ts",
                    "src/api/types/**",
                    "src/test/**",
                    "src/main.tsx",
                ],
            },
        },
    }),
);
