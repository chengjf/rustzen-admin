import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const mocks = vi.hoisted(() => ({
    login: vi.fn(),
    navigate: vi.fn(),
    success: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
    createFileRoute: () => () => ({}),
    useNavigate: () => mocks.navigate,
}));

vi.mock("@/api", () => ({
    appMessage: {
        success: mocks.success,
    },
}));

vi.mock("@/api/auth", () => ({
    authAPI: {
        login: mocks.login,
    },
}));

import { LoginPage } from "./login";

beforeEach(() => {
    useAuthStore.setState({ token: null, userInfo: null });
    vi.spyOn(console, "error").mockImplementation(() => {});
    Object.defineProperty(window, "matchMedia", {
        writable: true,
        value: vi.fn().mockImplementation(() => ({
            matches: false,
            media: "",
            onchange: null,
            addListener: vi.fn(),
            removeListener: vi.fn(),
            addEventListener: vi.fn(),
            removeEventListener: vi.fn(),
            dispatchEvent: vi.fn(),
        })),
    });
});

afterEach(() => {
    useAuthStore.setState({ token: null, userInfo: null });
    vi.clearAllMocks();
});

describe("LoginPage", () => {
    it("submits credentials, stores login state, and navigates home", async () => {
        mocks.login.mockResolvedValue({
            token: "new-token",
            userInfo: mockUserInfo,
        });

        render(<LoginPage />);

        fireEvent.change(screen.getByPlaceholderText("用户名"), {
            target: { value: "superadmin" },
        });
        fireEvent.change(screen.getByPlaceholderText("密码"), {
            target: { value: "Admin@123" },
        });
        fireEvent.click(screen.getByRole("button", { name: "登 录" }));

        await waitFor(() => {
            expect(mocks.login).toHaveBeenCalledWith({
                username: "superadmin",
                password: "Admin@123",
            });
        });

        expect(useAuthStore.getState().token).toBe("new-token");
        expect(useAuthStore.getState().userInfo).toEqual(mockUserInfo);
        expect(mocks.success).toHaveBeenCalledWith("登录成功，正在跳转...");
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/", replace: true });
    });

    it("restores the submit button after login failure", async () => {
        mocks.login.mockRejectedValue(new Error("bad credentials"));

        render(<LoginPage />);

        fireEvent.change(screen.getByPlaceholderText("用户名"), {
            target: { value: "superadmin" },
        });
        fireEvent.change(screen.getByPlaceholderText("密码"), {
            target: { value: "Admin@123" },
        });
        fireEvent.click(screen.getByRole("button", { name: "登 录" }));

        await waitFor(() => {
            expect(mocks.login).toHaveBeenCalledTimes(1);
        });

        await waitFor(() => {
            expect(screen.getByRole("button", { name: "登 录" })).toBeEnabled();
        });
        expect(useAuthStore.getState().token).toBeNull();
        expect(mocks.navigate).not.toHaveBeenCalled();
    });

    it("shows validation messages when fields are empty", async () => {
        render(<LoginPage />);

        fireEvent.click(screen.getByRole("button", { name: "登 录" }));

        expect(await screen.findByText("请输入用户名")).toBeInTheDocument();
        expect(await screen.findByText("请输入密码")).toBeInTheDocument();
        expect(mocks.login).not.toHaveBeenCalled();
    });

    it("shows length validation messages for short credentials", async () => {
        render(<LoginPage />);

        fireEvent.change(screen.getByPlaceholderText("用户名"), {
            target: { value: "ab" },
        });
        fireEvent.change(screen.getByPlaceholderText("密码"), {
            target: { value: "12345" },
        });
        fireEvent.click(screen.getByRole("button", { name: "登 录" }));

        expect(await screen.findByText("用户名长度不能少于3位")).toBeInTheDocument();
        expect(await screen.findByText("密码长度不能少于6位")).toBeInTheDocument();
    });
});
