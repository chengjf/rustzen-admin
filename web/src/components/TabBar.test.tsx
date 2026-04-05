import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useTabStore } from "@/stores/useTabStore";

const mocks = vi.hoisted(() => ({
    navigate: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
    useNavigate: () => mocks.navigate,
}));

vi.mock("antd", () => ({
    Dropdown: ({
        children,
        menu,
        onOpenChange,
    }: {
        children: React.ReactNode;
        menu?: { items?: Array<{ key?: React.Key; label?: React.ReactNode; disabled?: boolean; onClick?: () => void }> };
        onOpenChange?: (open: boolean) => void;
    }) => (
        <div>
            <div onMouseEnter={() => onOpenChange?.(true)}>{children}</div>
            {menu?.items?.map((item) =>
                item && "label" in item ? (
                    <button
                        key={String(item.key)}
                        disabled={item.disabled}
                        onClick={() => item.onClick?.()}
                    >
                        {item.label}
                    </button>
                ) : null,
            )}
        </div>
    ),
}));

import { TabBar } from "./TabBar";

beforeEach(() => {
    act(() => {
        useTabStore.setState({
            tabs: [
                { path: "/", title: "首页", closable: false },
                { path: "/system/user", title: "用户管理", closable: true },
                { path: "/system/role", title: "角色管理", closable: true },
            ],
            activeKey: "/system/user",
        });
    });
});

afterEach(() => {
    act(() => {
        useTabStore.setState({
            tabs: [{ path: "/", title: "首页", closable: false }],
            activeKey: "/",
        });
    });
    vi.clearAllMocks();
});

describe("TabBar", () => {
    it("navigates when a tab is clicked", () => {
        render(<TabBar />);

        act(() => {
            fireEvent.click(screen.getByText("角色管理"));
        });

        expect(useTabStore.getState().activeKey).toBe("/system/role");
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/system/role" });
    });

    it("navigates to the fallback tab after closing the active tab", () => {
        render(<TabBar />);

        act(() => {
            fireEvent.click(screen.getAllByRole("img", { name: "close" })[0]);
        });

        expect(useTabStore.getState().activeKey).toBe("/");
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/" });
    });

    it("closes all closable tabs from the context menu", () => {
        render(<TabBar />);

        act(() => {
            fireEvent.mouseEnter(screen.getByText("用户管理"));
            fireEvent.click(screen.getAllByRole("button", { name: "关闭全部" })[1]);
        });

        expect(useTabStore.getState().tabs).toEqual([
            { path: "/", title: "首页", closable: false },
        ]);
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/" });
    });

    it("reloads only the active tab", () => {
        const onReload = vi.fn();
        render(<TabBar onReload={onReload} />);

        expect(screen.getAllByRole("button", { name: "刷新当前" })[1]).toBeEnabled();
        act(() => {
            fireEvent.click(screen.getAllByRole("button", { name: "刷新当前" })[1]);
        });

        expect(onReload).toHaveBeenCalledTimes(1);
    });
});
