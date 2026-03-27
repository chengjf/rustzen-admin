import { apiRequest, proTableRequest } from "@/api";
import type { CreateUserDto } from "@/api/types/CreateUserDto";
import type { OptionItem } from "@/api/types/OptionItem";
import type { ResetPasswordResp } from "@/api/types/ResetPasswordResp";
import type { UpdateUserPasswordPayload } from "@/api/types/UpdateUserPasswordPayload";
import type { UpdateUserPayload } from "@/api/types/UpdateUserPayload";
import type { UpdateUserStatusPayload } from "@/api/types/UpdateUserStatusPayload";
import type { UserItemResp } from "@/api/types/UserItemResp";
import type { UserQuery } from "@/api/types/UserQuery";

/**
 * 用户管理API服务
 */
export const userAPI = {
    getTableData: (params?: Partial<UserQuery>) =>
        proTableRequest<UserItemResp, Partial<UserQuery>>({
            url: "/api/system/users",
            params,
        }),

    create: (data: CreateUserDto) =>
        apiRequest<UserItemResp, CreateUserDto>({
            url: "/api/system/users",
            method: "POST",
            params: data,
        }),

    update: (id: number, data: UpdateUserPayload) =>
        apiRequest<UserItemResp, UpdateUserPayload>({
            url: `/api/system/users/${id}`,
            method: "PUT",
            params: data,
        }),

    delete: (id: number) => apiRequest<void>({ url: `/api/system/users/${id}`, method: "DELETE" }),

    updateStatus: (id: number, data: UpdateUserStatusPayload) =>
        apiRequest<void, UpdateUserStatusPayload>({
            url: `/api/system/users/${id}/status`,
            method: "PUT",
            params: data,
        }),

    resetPassword: (id: number) =>
        apiRequest<ResetPasswordResp, UpdateUserPasswordPayload>({
            url: `/api/system/users/${id}/password`,
            method: "PUT",
            body: JSON.stringify({} as UpdateUserPasswordPayload),
        }),

    getStatusOptions: () =>
        apiRequest<OptionItem<string>[]>({
            url: "/api/system/users/status-options",
        }),
};
