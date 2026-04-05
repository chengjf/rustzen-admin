import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tanstack/react-router", () => ({
    createFileRoute: () => () => ({}),
    Link: ({ to, children }: { to: string; children?: React.ReactNode }) => (
        <a href={to}>{children}</a>
    ),
}));

import { RouteComponent } from "./403";

describe("403 page", () => {
    it("renders the forbidden message and a link back home", () => {
        render(<RouteComponent />);

        expect(screen.getByText("403")).toBeInTheDocument();
        expect(screen.getByText("访问被拒绝")).toBeInTheDocument();
        expect(screen.getByRole("link", { name: "返回首页" })).toHaveAttribute("href", "/");
    });
});
