import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tanstack/react-query-devtools", () => ({
    ReactQueryDevtools: ({ buttonPosition }: { buttonPosition?: string }) => (
        <div>{buttonPosition}</div>
    ),
}));

vi.mock("@tanstack/react-router-devtools", () => ({
    TanStackRouterDevtools: () => <div>router-devtools</div>,
}));

import { TanStackDevtoolsLayout } from "./layout";

describe("TanStackDevtoolsLayout", () => {
    it("renders both query and router devtools", () => {
        render(<TanStackDevtoolsLayout />);

        expect(screen.getByText("bottom-right")).toBeInTheDocument();
        expect(screen.getByText("router-devtools")).toBeInTheDocument();
    });
});
