import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
    apiRequest: vi.fn(),
}));

vi.mock("@/api", () => ({
    apiRequest: mocks.apiRequest,
}));

import { authAPI } from "./index";

describe("authAPI", () => {
    it("wraps login/logout/me/change-password endpoints with expected options", () => {
        authAPI.login({ username: "admin", password: "secret" });
        authAPI.logout();
        authAPI.getUserInfo();
        authAPI.changePassword({
            oldPassword: "old-secret",
            newPassword: "new-secret",
            confirmPassword: "new-secret",
        });

        expect(mocks.apiRequest).toHaveBeenCalledWith({
            url: "/api/auth/login",
            method: "POST",
            params: { username: "admin", password: "secret" },
            skipSuccessMsg: true,
        });
        expect(mocks.apiRequest).toHaveBeenCalledWith({ url: "/api/auth/logout" });
        expect(mocks.apiRequest).toHaveBeenCalledWith({ url: "/api/auth/me" });
        expect(mocks.apiRequest).toHaveBeenCalledWith({
            url: "/api/auth/self/password",
            method: "PUT",
            params: {
                oldPassword: "old-secret",
                newPassword: "new-secret",
                confirmPassword: "new-secret",
            },
            skipSuccessMsg: true,
        });
    });
});
