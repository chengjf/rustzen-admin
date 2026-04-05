import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "@/stores/useAuthStore";

vi.mock("@/api", () => ({
    MessageContent: () => null,
}));

vi.mock("@/api/auth", () => ({
    authAPI: {
        getUserInfo: () => Promise.resolve(mockUserInfo),
    },
}));

vi.mock("@/integrations/tanstack-query/layout", () => ({
    TanStackDevtoolsLayout: () => null,
}));

vi.mock("@/layouts/BasicLayout", () => ({
    BasicLayout: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

import { rootBeforeLoad } from "./__root";

const runBeforeLoad = (pathname: string) => rootBeforeLoad({ location: { pathname } });
const expectRedirectTo = (pathname: string, target: string) => {
    try {
        runBeforeLoad(pathname);
    } catch (error) {
        expect(error).toMatchObject({
            options: {
                to: target,
            },
        });
        return;
    }

    throw new Error(`Expected redirect to ${target}`);
};

beforeEach(() => {
    vi.spyOn(console, "log").mockImplementation(() => {});
    useAuthStore.setState({ token: null, userInfo: null });
});

afterEach(() => {
    useAuthStore.setState({ token: null, userInfo: null });
});

describe("rootBeforeLoad", () => {
    it("redirects unauthenticated users to login", () => {
        expectRedirectTo("/system/user", "/login");
    });

    it("allows the login page when unauthenticated", () => {
        expect(runBeforeLoad("/login")).toBeNull();
    });

    it("redirects authenticated users away from login", () => {
        useAuthStore.setState({ token: "token", userInfo: mockUserInfo });

        expectRedirectTo("/login", "/");
    });

    it("redirects to 403 when the user lacks menu permission", () => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, permissions: ["system:role:list"] },
        });

        expectRedirectTo("/system/user", "/403");
    });

    it("allows protected pages when the user has permission", () => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, permissions: ["system:user:list"] },
        });

        expect(runBeforeLoad("/system/user")).toBeUndefined();
    });
});
