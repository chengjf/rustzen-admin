import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "./useAuthStore";

// Reset Zustand state (and the persisted localStorage entry) before each test
beforeEach(() => {
    useAuthStore.setState({ userInfo: null, token: null });
});

afterEach(() => {
    useAuthStore.setState({ userInfo: null, token: null });
});

describe("useAuthStore – handleLogin / clearAuth", () => {
    it("handleLogin stores token and userInfo", () => {
        useAuthStore.getState().handleLogin("test-token", mockUserInfo);

        const { token, userInfo } = useAuthStore.getState();
        expect(token).toBe("test-token");
        expect(userInfo).toEqual(mockUserInfo);
    });

    it("clearAuth removes token and userInfo", () => {
        useAuthStore.getState().handleLogin("test-token", mockUserInfo);
        useAuthStore.getState().clearAuth();

        const { token, userInfo } = useAuthStore.getState();
        expect(token).toBeNull();
        expect(userInfo).toBeNull();
    });
});

describe("useAuthStore – updateAvatar", () => {
    it("updates avatarUrl when userInfo is set", () => {
        useAuthStore.getState().handleLogin("tok", mockUserInfo);
        useAuthStore.getState().updateAvatar("https://cdn.example.com/avatar.png");

        expect(useAuthStore.getState().userInfo?.avatarUrl).toBe(
            "https://cdn.example.com/avatar.png",
        );
    });

    it("does nothing when userInfo is null", () => {
        useAuthStore.getState().updateAvatar("https://cdn.example.com/avatar.png");
        // Should not throw, state stays null
        expect(useAuthStore.getState().userInfo).toBeNull();
    });
});

describe("useAuthStore – checkPermissions", () => {
    const setPerms = (permissions: string[]) => {
        useAuthStore.setState({ userInfo: { ...mockUserInfo, permissions } });
    };

    it("returns false when no userInfo", () => {
        expect(useAuthStore.getState().checkPermissions("system:user:list")).toBe(false);
    });

    it("returns true for wildcard '*' user", () => {
        setPerms(["*"]);
        expect(useAuthStore.getState().checkPermissions("system:user:list")).toBe(true);
        expect(useAuthStore.getState().checkPermissions("anything:else")).toBe(true);
    });

    it("returns true for exact permission match", () => {
        setPerms(["system:user:list"]);
        expect(useAuthStore.getState().checkPermissions("system:user:list")).toBe(true);
    });

    it("returns false when permission not in list", () => {
        setPerms(["system:role:list"]);
        expect(useAuthStore.getState().checkPermissions("system:user:list")).toBe(false);
    });

    it("returns true for prefix wildcard (system:user:*)", () => {
        setPerms(["system:user:*"]);
        expect(useAuthStore.getState().checkPermissions("system:user:list")).toBe(true);
        expect(useAuthStore.getState().checkPermissions("system:user:delete")).toBe(true);
    });

    it("returns true for broader prefix (system:*)", () => {
        setPerms(["system:*"]);
        expect(useAuthStore.getState().checkPermissions("system:user:list")).toBe(true);
    });

    it("prefix wildcard does not grant cross-domain access", () => {
        setPerms(["system:user:*"]);
        expect(useAuthStore.getState().checkPermissions("dashboard:stats:view")).toBe(false);
    });
});

describe("useAuthStore – checkMenuPermissions", () => {
    it("converts path to permission code (list page)", () => {
        useAuthStore.setState({
            userInfo: { ...mockUserInfo, permissions: ["system:user:list"] },
        });
        expect(useAuthStore.getState().checkMenuPermissions("/system/user")).toBe(true);
    });

    it("keeps create pages as create permissions", () => {
        useAuthStore.setState({
            userInfo: { ...mockUserInfo, permissions: ["system:user:create"] },
        });
        expect(useAuthStore.getState().checkMenuPermissions("/system/user/create")).toBe(true);
    });

    it("normalizes edit pages with numeric ids", () => {
        useAuthStore.setState({
            userInfo: { ...mockUserInfo, permissions: ["system:user:edit"] },
        });
        expect(useAuthStore.getState().checkMenuPermissions("/system/user/42/edit")).toBe(true);
    });

    it("normalizes detail pages with numeric ids", () => {
        useAuthStore.setState({
            userInfo: { ...mockUserInfo, permissions: ["system:user:detail"] },
        });
        expect(useAuthStore.getState().checkMenuPermissions("/system/user/42/detail")).toBe(
            true,
        );
    });
});
