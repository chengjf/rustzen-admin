import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

import { getMenuData } from "./index";

beforeEach(() => {
    useAuthStore.setState({ token: "token", userInfo: mockUserInfo });
});

afterEach(() => {
    useAuthStore.setState({ token: null, userInfo: null });
});

describe("getMenuData", () => {
    it("filters menus by permission", () => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, permissions: ["system:user:list", "system:log:list"] },
        });

        const menus = getMenuData();

        expect(menus).toHaveLength(1);
        expect(menus[0].children?.map((item) => item.path)).toEqual([
            "/system/user",
            "/system/log",
        ]);
    });

    it("hides parent groups when no child menus are accessible", () => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, permissions: [] },
        });

        expect(getMenuData()).toEqual([]);
    });
});
