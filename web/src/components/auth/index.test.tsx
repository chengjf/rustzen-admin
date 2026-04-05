import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "@/stores/useAuthStore";

const mocks = vi.hoisted(() => ({
    confirm: vi.fn(),
    onConfirm: vi.fn(() => Promise.resolve()),
    onCancel: vi.fn(() => Promise.resolve()),
}));

vi.mock("antd", () => ({
    Popconfirm: ({
        children,
        onConfirm,
    }: {
        children?: React.ReactNode;
        onConfirm?: () => Promise<void>;
    }) => <button onClick={() => void onConfirm?.()}>{children}</button>,
}));

vi.mock("@/api", () => ({
    appModal: {
        confirm: mocks.confirm,
    },
}));

import { AuthConfirm, AuthPopconfirm, AuthWrap } from ".";

beforeEach(() => {
    act(() => {
        useAuthStore.setState({ userInfo: null, token: null });
    });
});

afterEach(() => {
    act(() => {
        useAuthStore.setState({ userInfo: null, token: null });
    });
});

describe("AuthWrap", () => {
    it("renders fallback without permission and updates when permission is granted", () => {
        render(
            <AuthWrap code="system:user:create" fallback={<span>denied</span>}>
                <span>allowed</span>
            </AuthWrap>,
        );

        expect(screen.getByText("denied")).toBeInTheDocument();
        expect(screen.queryByText("allowed")).not.toBeInTheDocument();

        act(() => {
            useAuthStore.setState({
                userInfo: { ...mockUserInfo, permissions: ["system:user:create"] },
            });
        });

        expect(screen.getByText("allowed")).toBeInTheDocument();
        expect(screen.queryByText("denied")).not.toBeInTheDocument();
    });

    it("invokes appModal.confirm for AuthConfirm when permitted", () => {
        act(() => {
            useAuthStore.setState({
                userInfo: { ...mockUserInfo, permissions: ["system:user:delete"] },
            });
        });

        render(
            <AuthConfirm
                code="system:user:delete"
                title="确认删除"
                description="删除后不可恢复"
                onConfirm={mocks.onConfirm}
                onCancel={mocks.onCancel}
            >
                删除
            </AuthConfirm>,
        );

        fireEvent.click(screen.getByText("删除"));

        expect(mocks.confirm).toHaveBeenCalledWith(
            expect.objectContaining({
                title: "确认删除",
                content: "删除后不可恢复",
                onOk: mocks.onConfirm,
                onCancel: mocks.onCancel,
            }),
        );
    });

    it("triggers popconfirm actions for AuthPopconfirm when permitted", async () => {
        act(() => {
            useAuthStore.setState({
                userInfo: { ...mockUserInfo, permissions: ["system:user:delete"] },
            });
        });

        render(
            <AuthPopconfirm
                code="system:user:delete"
                title="确认删除"
                onConfirm={mocks.onConfirm}
            >
                删除用户
            </AuthPopconfirm>,
        );

        fireEvent.click(screen.getByRole("button", { name: "删除用户" }));

        expect(mocks.onConfirm).toHaveBeenCalledTimes(1);
    });
});
