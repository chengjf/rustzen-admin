import { fileURLToPath, URL } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite-plus";

const VENDOR_CHUNK_GROUPS: Array<[string, string[]]> = [
    ["react-vendor", ["react", "react-dom", "scheduler"]],
    ["router-vendor", ["@tanstack/react-router"]],
    [
        "query-vendor",
        ["@tanstack/react-query", "@tanstack/react-query-devtools", "@tanstack/query-core"],
    ],
    ["antd-vendor", ["antd", "@ant-design/icons"]],
    ["procomponents-vendor", ["@ant-design/pro-components", "@ant-design/pro-layout"]],
    ["emotion-vendor", ["@emotion/react", "@emotion/cache", "@emotion/serialize"]],
];

const getVendorChunkName = (id: string) => {
    if (!id.includes("node_modules")) {
        return undefined;
    }

    const matchedChunk = VENDOR_CHUNK_GROUPS.find(([, packages]) =>
        packages.some((pkg) => id.includes(`/node_modules/${pkg}/`)),
    );

    if (matchedChunk) {
        return matchedChunk[0];
    }

    const packageNameMatch = id.match(
        /\/node_modules\/(?:\.pnpm\/[^/]+\/node_modules\/)?((?:@[^/]+\/)?[^/]+)/,
    );
    const packageName = packageNameMatch?.[1];

    if (!packageName) {
        return "vendor";
    }

    return packageName
        .replace(/^@/, "")
        .replace(/[\\/]/g, "-")
        .replace(/[^a-zA-Z0-9-_]/g, "_");
};

// https://vite.dev/config/
export default defineConfig({
    lint: { options: { typeAware: true, typeCheck: true } },
    fmt: { sortImports: {} },
    staged: {
        "*": "vp check --fix",
    },
    build: {
        chunkSizeWarningLimit: 1500,
        rollupOptions: {
            output: {
                manualChunks: getVendorChunkName,
            },
        },
    },
    plugins: [
        tanstackRouter({
            autoCodeSplitting: true,
            routeFileIgnorePattern: "\\.(test|spec)\\.(ts|tsx)$",
        }),
        viteReact({ jsxImportSource: "@emotion/react" }),
        tailwindcss(),
    ],
    resolve: {
        alias: {
            "@": fileURLToPath(new URL("./src", import.meta.url)),
        },
        tsconfigPaths: true,
    },
    server: {
        port: 8008,
        open: false,
        proxy: {
            "/api": {
                target: "http://localhost:8000",
                changeOrigin: true,
            },
            "/uploads": {
                target: "http://localhost:8000",
                changeOrigin: true,
            },
        },
    },
});
