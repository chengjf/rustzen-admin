import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const TestAction = ({
    children,
    code: _code,
    hidden,
}: {
    children: React.ReactNode;
    code?: string;
    hidden?: boolean;
}) => (
    <button hidden={hidden}>{children}</button>
);

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
                    <TestAction key="edit" code="system:user:update">
                        编辑
                    </TestAction>,
                    <TestAction key="delete" code="system:user:delete">
                        删除
                    </TestAction>,
                    <TestAction key="hidden" hidden>
                        隐藏
                    </TestAction>,
                    <TestAction key="plain">查看</TestAction>,
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
                    <TestAction key="delete" code="system:user:delete">
                        删除
                    </TestAction>,
                    <TestAction key="hidden" hidden>
                        隐藏
                    </TestAction>,
                ]}
            </MoreButton>,
        );

        expect(container).toBeEmptyDOMElement();
    });
});
