import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "@/stores/useAuthStore";

const mocks = vi.hoisted(() => ({
    useQuery: vi.fn(),
}));

vi.mock("@tanstack/react-query", () => ({
    useQuery: mocks.useQuery,
}));

vi.mock("@tanstack/react-router", () => ({
    createRootRoute: () => ({}),
    Navigate: () => null,
    Outlet: () => <div>outlet</div>,
    redirect: vi.fn(),
}));

vi.mock("antd", () => ({
    App: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
    ConfigProvider: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock("antd/locale/en_US", () => ({
    default: {},
}));

vi.mock("@/api", () => ({
    MessageContent: () => <div>message-content</div>,
}));

vi.mock("@/api/auth", () => ({
    authAPI: {
        getUserInfo: vi.fn(),
    },
}));

vi.mock("@/integrations/tanstack-query/layout", () => ({
    TanStackDevtoolsLayout: () => <div>devtools</div>,
}));

vi.mock("@/layouts/BasicLayout", () => ({
    BasicLayout: ({
        children,
        hidden,
    }: {
        children?: React.ReactNode;
        hidden?: boolean;
    }) => <div>{hidden ? "hidden-layout" : "visible-layout"}{children}</div>,
}));

import { RootLayout } from "./__root";

beforeEach(() => {
    vi.spyOn(console, "error").mockImplementation(() => {});
});

afterEach(() => {
    useAuthStore.setState({ token: null, userInfo: null });
    vi.clearAllMocks();
});

describe("RootLayout", () => {
    it("updates user info when the query succeeds", () => {
        useAuthStore.setState({ token: "token", userInfo: null });
        mocks.useQuery.mockReturnValue({
            data: mockUserInfo,
            error: null,
        });

        render(<RootLayout />);

        expect(useAuthStore.getState().userInfo).toEqual(mockUserInfo);
        expect(screen.getByText("visible-layout")).toBeInTheDocument();
    });

    it("renders the hidden layout when no token is present", () => {
        useAuthStore.setState({ token: null, userInfo: null });
        mocks.useQuery.mockReturnValue({
            data: undefined,
            error: null,
        });

        render(<RootLayout />);

        expect(screen.getByText("hidden-layout")).toBeInTheDocument();
    });

    it("logs query errors without updating user info", () => {
        useAuthStore.setState({ token: "token", userInfo: null });
        const error = new Error("load failed");
        mocks.useQuery.mockReturnValue({
            data: undefined,
            error,
        });

        render(<RootLayout />);

        expect(console.error).toHaveBeenCalledWith("[UserInfo Load Error]:", error);
        expect(useAuthStore.getState().userInfo).toBeNull();
    });
});
