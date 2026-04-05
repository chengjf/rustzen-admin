import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
    back: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
    createFileRoute: () => () => ({}),
    Link: ({
        to,
        children,
    }: {
        to: string;
        children?: React.ReactNode;
    }) => <a href={to}>{children}</a>,
}));

import { RouteComponent } from "./404";

describe("404 page", () => {
    it("renders the not found message, home link, and back button", () => {
        vi.spyOn(window.history, "back").mockImplementation(mocks.back);

        render(<RouteComponent />);

        expect(screen.getByText("404")).toBeInTheDocument();
        expect(screen.getByText("页面走丢了")).toBeInTheDocument();
        expect(screen.getByRole("link", { name: "返回首页" })).toHaveAttribute("href", "/");

        fireEvent.click(screen.getByRole("button", { name: "返回上一页" }));
        expect(mocks.back).toHaveBeenCalledTimes(1);
    });
});
