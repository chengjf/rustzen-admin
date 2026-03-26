import { apiRequest, proTableRequest } from "@/api";
import type {
    UserItemResp,
    UserQuery,
    CreateUserDto,
    UpdateUserPayload,
    UpdateUserPasswordPayload,
    UpdateUserStatusPayload,
    OptionItem,
} from "@/api/types";

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

    resetPassword: (id: number, data: UpdateUserPasswordPayload) =>
        apiRequest<void, UpdateUserPasswordPayload>({
            url: `/api/system/users/${id}/password`,
            method: "PUT",
            params: data,
        }),

    getStatusOptions: () =>
        apiRequest<OptionItem<string>[]>({
            url: "/api/system/users/status-options",
        }),
};
