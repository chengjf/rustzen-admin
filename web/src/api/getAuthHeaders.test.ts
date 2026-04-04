import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "@/stores/useAuthStore";
import { getAuthHeaders } from "@/api";

beforeEach(() => {
    useAuthStore.setState({ userInfo: null, token: null });
});

afterEach(() => {
    useAuthStore.setState({ userInfo: null, token: null });
});

describe("getAuthHeaders", () => {
    it("returns empty object when not authenticated", () => {
        expect(getAuthHeaders()).toEqual({});
    });

    it("returns Authorization header when token is set", () => {
        useAuthStore.setState({ token: "my-secret-token", userInfo: mockUserInfo });
        expect(getAuthHeaders()).toEqual({ Authorization: "Bearer my-secret-token" });
    });

    it("reflects token changes dynamically", () => {
        useAuthStore.setState({ token: "token-v1", userInfo: mockUserInfo });
        expect(getAuthHeaders().Authorization).toBe("Bearer token-v1");

        useAuthStore.setState({ token: "token-v2" });
        expect(getAuthHeaders().Authorization).toBe("Bearer token-v2");

        useAuthStore.getState().clearAuth();
        expect(getAuthHeaders()).toEqual({});
    });
});

/**
 * MSW fetch interception tests.
 *
 * These tests verify that MSW intercepts fetch calls at the network level
 * and returns the expected mock responses, confirming that the handlers.ts
 * setup works correctly for integration testing.
 */
describe("MSW handler interception (fetch-level)", () => {
    it("intercepts POST /api/auth/login and returns mock token", async () => {
        const response = await fetch("/api/auth/login", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ username: "superadmin", password: "Admin@123" }),
        });
        const data = await response.json();

        expect(response.ok).toBe(true);
        expect(data.code).toBe(0);
        expect(data.data.token).toBe("mock-token-abc123");
        expect(data.data.userInfo.username).toBe("superadmin");
    });

    it("intercepts GET /api/auth/me and returns user info", async () => {
        const response = await fetch("/api/auth/me");
        const data = await response.json();

        expect(response.ok).toBe(true);
        expect(data.code).toBe(0);
        expect(data.data.isSystem).toBe(true);
        expect(data.data.permissions).toContain("*");
    });

    it("intercepts DELETE /api/auth/logout", async () => {
        const response = await fetch("/api/auth/logout", { method: "DELETE" });
        const data = await response.json();

        expect(response.ok).toBe(true);
        expect(data.code).toBe(0);
    });
});
