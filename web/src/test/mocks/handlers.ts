import { HttpResponse, http } from "msw";

import type { UserInfoResp } from "@/api/types/UserInfoResp";

/** Reusable mock user for tests */
export const mockUserInfo: UserInfoResp = {
    id: 1,
    username: "superadmin",
    realName: "超级管理员",
    email: "superadmin@example.com",
    avatarUrl: null,
    isSystem: true,
    permissions: ["*"],
};

export const handlers = [
    // POST /api/auth/login — success
    http.post("/api/auth/login", () => {
        return HttpResponse.json({
            code: 0,
            message: "操作成功",
            data: { token: "mock-token-abc123", userInfo: mockUserInfo },
        });
    }),

    // GET /api/auth/me — returns current user info
    http.get("/api/auth/me", () => {
        return HttpResponse.json({
            code: 0,
            message: "操作成功",
            data: mockUserInfo,
        });
    }),

    // GET /api/auth/logout
    http.get("/api/auth/logout", () => {
        return HttpResponse.json({ code: 0, message: "操作成功", data: null });
    }),
];
