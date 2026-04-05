import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("antd", () => ({
    Button: ({ children, onClick }: { children: React.ReactNode; onClick?: () => void }) => (
        <button onClick={onClick}>{children}</button>
    ),
    Result: ({
        title,
        subTitle,
        extra,
    }: {
        title?: React.ReactNode;
        subTitle?: React.ReactNode;
        extra?: React.ReactNode;
    }) => (
        <div>
            <div>{title}</div>
            <div>{subTitle}</div>
            {extra}
        </div>
    ),
}));

import { ErrorBoundary } from "./error-boundary";

const Thrower = ({ shouldThrow }: { shouldThrow: boolean }) => {
    if (shouldThrow) {
        throw new Error("boom");
    }

    return <div>safe content</div>;
};

describe("ErrorBoundary", () => {
    it("renders a fallback when a child throws and allows a clean remount", () => {
        vi.spyOn(console, "error").mockImplementation(() => {});
        const view = render(
            <ErrorBoundary>
                <Thrower shouldThrow />
            </ErrorBoundary>,
        );

        expect(screen.getByText("出错了")).toBeInTheDocument();
        expect(screen.getByText("抱歉，页面遇到了一个错误。请尝试刷新页面。")).toBeInTheDocument();

        fireEvent.click(screen.getByRole("button", { name: "重试" }));

        view.unmount();

        render(
            <ErrorBoundary>
                <Thrower shouldThrow={false} />
            </ErrorBoundary>,
        );

        expect(screen.getByText("safe content")).toBeInTheDocument();
    });
});
