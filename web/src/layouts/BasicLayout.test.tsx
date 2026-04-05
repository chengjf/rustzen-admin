import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "@/stores/useAuthStore";
import { useTabStore } from "@/stores/useTabStore";

const mocks = vi.hoisted(() => ({
    logout: vi.fn(() => Promise.resolve()),
    navigate: vi.fn(() => Promise.resolve()),
    success: vi.fn(),
    currentPath: "/system/user",
}));

vi.mock("@tanstack/react-router", () => ({
    Link: ({ children }: { children: React.ReactNode }) => <>{children}</>,
    useLocation: () => ({ pathname: mocks.currentPath }),
    useRouter: () => ({ navigate: mocks.navigate }),
}));

vi.mock("@ant-design/pro-components", () => ({
    ProLayout: ({
        children,
        avatarProps,
        title,
        onMenuHeaderClick,
    }: {
        children: React.ReactNode;
        avatarProps?: {
            render?: (_props: unknown, dom: React.ReactNode) => React.ReactNode;
        };
        title?: string;
        onMenuHeaderClick?: () => void;
    }) => (
        <div>
            <button onClick={onMenuHeaderClick}>{title}</button>
            <div data-testid="avatar-area">
                {avatarProps?.render?.({}, <span>avatar</span>)}
            </div>
            {children}
        </div>
    ),
}));

vi.mock("antd", () => ({
    Dropdown: ({
        children,
        menu,
    }: {
        children: React.ReactNode;
        menu?: { items?: Array<{ key?: React.Key; label?: React.ReactNode; onClick?: () => void }> };
    }) => (
        <div>
            {children}
            {menu?.items?.map((item) =>
                item && "label" in item ? (
                    <button key={String(item.key)} onClick={() => item.onClick?.()}>
                        {item.label}
                    </button>
                ) : null,
            )}
        </div>
    ),
}));

vi.mock("@/api", () => ({
    appMessage: {
        success: mocks.success,
    },
}));

vi.mock("@/api/auth", () => ({
    authAPI: {
        logout: mocks.logout,
    },
}));

vi.mock("@/components/TabBar", () => ({
    TabBar: () => <div data-testid="tab-bar" />,
}));

vi.mock("@/components/user/ChangePasswordModal", () => ({
    ChangePasswordModal: () => <span>修改密码</span>,
}));

vi.mock("@/components/user/index", () => ({
    UserProfileModal: () => <span>个人信息</span>,
}));

vi.mock("@/layouts", () => ({
    getMenuData: () => [],
}));

import { BasicLayout } from "./BasicLayout";

beforeEach(() => {
    act(() => {
        useAuthStore.setState({ token: "token", userInfo: mockUserInfo });
        useTabStore.setState({
            tabs: [
                { path: "/", title: "首页", closable: false },
                { path: "/system/user", title: "用户管理", closable: true },
            ],
            activeKey: "/system/user",
        });
    });
});

afterEach(() => {
    act(() => {
        useAuthStore.setState({ token: null, userInfo: null });
        useTabStore.setState({
            tabs: [{ path: "/", title: "首页", closable: false }],
            activeKey: "/",
        });
    });
    vi.clearAllMocks();
});

describe("BasicLayout logout flow", () => {
    it("adds the current route tab on mount", () => {
        useTabStore.setState({
            tabs: [{ path: "/", title: "首页", closable: false }],
            activeKey: "/",
        });

        render(
            <BasicLayout>
                <div>content</div>
            </BasicLayout>,
        );

        expect(useTabStore.getState().tabs).toEqual([
            { path: "/", title: "首页", closable: false },
            { path: "/system/user", title: "用户管理", closable: true },
        ]);
        expect(useTabStore.getState().activeKey).toBe("/system/user");
    });

    it("navigates home when the header is clicked", () => {
        render(
            <BasicLayout>
                <div>content</div>
            </BasicLayout>,
        );

        fireEvent.click(screen.getByRole("button", { name: "Rustzen Admin" }));

        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/" });
    });

    it("shows realName and falls back to username when realName is missing", () => {
        const { rerender } = render(
            <BasicLayout>
                <div>content</div>
            </BasicLayout>,
        );

        expect(screen.getByText("超级管理员")).toBeInTheDocument();

        act(() => {
            useAuthStore.setState({
                token: "token",
                userInfo: { ...mockUserInfo, realName: null },
            });
        });

        rerender(
            <BasicLayout>
                <div>content</div>
            </BasicLayout>,
        );

        expect(screen.getByText("superadmin")).toBeInTheDocument();
    });

    it("logs out, clears auth state and tabs, then navigates to login", async () => {
        render(
            <BasicLayout>
                <div>content</div>
            </BasicLayout>,
        );

        fireEvent.click(screen.getByRole("button", { name: "退出登录" }));

        await waitFor(() => {
            expect(mocks.logout).toHaveBeenCalledTimes(1);
        });

        expect(useAuthStore.getState().token).toBeNull();
        expect(useAuthStore.getState().userInfo).toBeNull();
        expect(useTabStore.getState().tabs).toEqual([
            { path: "/", title: "首页", closable: false },
        ]);
        expect(useTabStore.getState().activeKey).toBe("/");
        expect(mocks.success).toHaveBeenCalledWith("退出登录成功");
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/login" });
    });
});
