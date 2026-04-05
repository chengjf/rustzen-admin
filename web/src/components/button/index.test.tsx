import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

vi.mock("antd", () => ({
    Dropdown: ({
        children,
        menu,
    }: {
        children: React.ReactNode;
        menu?: { items?: Array<{ key?: React.Key; label?: React.ReactNode }> };
    }) => (
        <div>
            {children}
            {menu?.items?.map((item) => (
                <div key={String(item.key)}>{item.label}</div>
            ))}
        </div>
    ),
}));

import { MoreButton } from ".";

beforeEach(() => {
    useAuthStore.setState({ token: "token", userInfo: mockUserInfo });
});

afterEach(() => {
    useAuthStore.setState({ token: null, userInfo: null });
});

describe("MoreButton", () => {
    it("filters actions by permission and hidden state", () => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, permissions: ["system:user:update"] },
        });

        render(
            <MoreButton>
                {[
                    <button key="edit" code="system:user:update">
                        编辑
                    </button>,
                    <button key="delete" code="system:user:delete">
                        删除
                    </button>,
                    <button key="hidden" hidden>
                        隐藏
                    </button>,
                    <button key="plain">查看</button>,
                ]}
            </MoreButton>,
        );

        expect(screen.getByText("更多")).toBeInTheDocument();
        expect(screen.getByText("编辑")).toBeInTheDocument();
        expect(screen.getByText("查看")).toBeInTheDocument();
        expect(screen.queryByText("删除")).not.toBeInTheDocument();
        expect(screen.queryByText("隐藏")).not.toBeInTheDocument();
    });

    it("renders nothing when no actions are visible", () => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, permissions: [] },
        });

        const { container } = render(
            <MoreButton>
                {[
                    <button key="delete" code="system:user:delete">
                        删除
                    </button>,
                    <button key="hidden" hidden>
                        隐藏
                    </button>,
                ]}
            </MoreButton>,
        );

        expect(container).toBeEmptyDOMElement();
    });
});
