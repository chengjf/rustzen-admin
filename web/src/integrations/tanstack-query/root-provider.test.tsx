import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
    queryClientProvider: vi.fn(),
}));

vi.mock("@tanstack/react-query", () => ({
    QueryClient: class QueryClient {
        options: any;
        constructor(options: any) {
            this.options = options;
        }
    },
    QueryClientProvider: ({
        children,
        client,
    }: {
        children?: React.ReactNode;
        client: { options?: any };
    }) => (
        <div>
            <div>{String(client.options?.defaultOptions?.queries?.retry)}</div>
            {children}
        </div>
    ),
}));

import { Provider, getContext } from "./root-provider";

describe("tanstack query root provider", () => {
    it("creates a query client with the expected default options", () => {
        const context = getContext();

        expect(context.queryClient.options.defaultOptions.queries.retry).toBe(0);
        expect(context.queryClient.options.defaultOptions.mutations.retry).toBe(false);
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
