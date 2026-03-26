import { apiRequest } from "@/api";
import type { LoginRequest, LoginResp, UserInfoResp } from "@/api/types";

/**
 * 认证相关API服务
 */
export const authAPI = {
    /**
     * 用户登录
     */
    login: (data: LoginRequest) =>
        apiRequest<LoginResp, LoginRequest>({
            url: "/api/auth/login",
            method: "POST",
            params: data,
        }),

    /**
     * 用户登出
     */
    logout: () => apiRequest<void>({ url: "/api/auth/logout" }),

    /**
     * 获取当前用户信息
     */
    getUserInfo: () => apiRequest<UserInfoResp>({ url: "/api/auth/me" }),
};
