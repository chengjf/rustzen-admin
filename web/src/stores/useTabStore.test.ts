import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useTabStore } from "./useTabStore";

// Reset to initial state before each test
beforeEach(() => {
    useTabStore.setState({ tabs: [{ path: "/", title: "首页", closable: false }], activeKey: "/" });
});

afterEach(() => {
    useTabStore.setState({ tabs: [{ path: "/", title: "首页", closable: false }], activeKey: "/" });
});

describe("useTabStore – addTab", () => {
    it("adds a new tab and activates it", () => {
        useTabStore.getState().addTab("/system/user", "用户管理");

        const { tabs, activeKey } = useTabStore.getState();
        expect(tabs).toHaveLength(2);
        expect(tabs[1].path).toBe("/system/user");
        expect(tabs[1].closable).toBe(true);
        expect(activeKey).toBe("/system/user");
    });

    it("does not duplicate tab on second add", () => {
        useTabStore.getState().addTab("/system/user", "用户管理");
        useTabStore.getState().addTab("/system/user", "用户管理");

        expect(useTabStore.getState().tabs).toHaveLength(2);
    });

    it("activates existing tab without adding duplicate", () => {
        useTabStore.getState().addTab("/system/user", "用户管理");
        useTabStore.getState().addTab("/system/role", "角色管理");
        // go back to user tab
        useTabStore.getState().addTab("/system/user", "用户管理");

        expect(useTabStore.getState().activeKey).toBe("/system/user");
        expect(useTabStore.getState().tabs).toHaveLength(3);
    });

    it("home tab is not closable", () => {
        const { tabs } = useTabStore.getState();
        expect(tabs[0].closable).toBe(false);
    });
});

describe("useTabStore – removeTab", () => {
    it("removes a tab and activates the previous one", () => {
        useTabStore.getState().addTab("/system/user", "用户管理");
        useTabStore.getState().addTab("/system/role", "角色管理");

        const { newActiveKey } = useTabStore.getState().removeTab("/system/role");

        expect(newActiveKey).toBe("/system/user");
        expect(useTabStore.getState().tabs).toHaveLength(2);
    });

    it("activates next tab when removing first of many", () => {
        useTabStore.getState().addTab("/system/user", "用户管理");
        useTabStore.getState().addTab("/system/role", "角色管理");
        // Make /system/user active then remove it
        useTabStore.getState().addTab("/system/user", "用户管理");

        const { newActiveKey } = useTabStore.getState().removeTab("/system/user");

        // idx of /system/user was 1 → Math.max(0, 1-1) = 0 → home tab
        expect(newActiveKey).toBe("/");
    });
});

describe("useTabStore – setActiveKey", () => {
    it("changes active key", () => {
        useTabStore.getState().addTab("/system/user", "用户管理");
        useTabStore.getState().setActiveKey("/");

        expect(useTabStore.getState().activeKey).toBe("/");
    });
});

describe("useTabStore – clearTabs", () => {
    it("resets to only the home tab", () => {
        useTabStore.getState().addTab("/system/user", "用户管理");
        useTabStore.getState().addTab("/system/role", "角色管理");
        useTabStore.getState().clearTabs();

        const { tabs, activeKey } = useTabStore.getState();
        expect(tabs).toHaveLength(1);
        expect(tabs[0].path).toBe("/");
        expect(activeKey).toBe("/");
    });
});
