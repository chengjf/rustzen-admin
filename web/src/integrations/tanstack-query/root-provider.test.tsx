import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tanstack/react-query", () => ({
    QueryClient: class QueryClient {
        private defaultOptions: {
            queries?: { retry?: number };
            mutations?: { retry?: boolean };
        };

        constructor(options: {
            defaultOptions: {
                queries?: { retry?: number };
                mutations?: { retry?: boolean };
            };
        }) {
            this.defaultOptions = options.defaultOptions;
        }

        getDefaultOptions() {
            return this.defaultOptions;
        }
    },
    QueryClientProvider: ({
        children,
        client,
    }: {
        children?: React.ReactNode;
        client: { getDefaultOptions: () => { queries?: { retry?: number } } };
    }) => (
        <div>
            <div>{String(client.getDefaultOptions().queries?.retry)}</div>
            {children}
        </div>
    ),
}));

import { Provider, getContext } from "./root-provider";

describe("tanstack query root provider", () => {
    it("creates a query client with the expected default options", () => {
        const context = getContext();

        expect(context.queryClient.getDefaultOptions().queries?.retry).toBe(0);
        expect(context.queryClient.getDefaultOptions().mutations?.retry).toBe(false);
    });

    it("renders children through QueryClientProvider", () => {
        const context = getContext();

        render(
            <Provider queryClient={context.queryClient}>
                <div>provider-child</div>
            </Provider>,
        );

        expect(screen.getByText("0")).toBeInTheDocument();
        expect(screen.getByText("provider-child")).toBeInTheDocument();
    });
});
