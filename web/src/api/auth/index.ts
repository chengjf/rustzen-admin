import { apiRequest } from "@/api";
import type { LoginRequest } from "@/api/types/LoginRequest";
import type { LoginResp } from "@/api/types/LoginResp";
import type { UserInfoResp } from "@/api/types/UserInfoResp";
import type { ChangePasswordPayload } from "../types/ChangePasswordPayload";

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


    changePassword: (data: ChangePasswordPayload) =>
        apiRequest<void, ChangePasswordPayload>({
            url: "/api/auth/self/password",
            method: "PUT",
            params: data,
        }),
    
};
