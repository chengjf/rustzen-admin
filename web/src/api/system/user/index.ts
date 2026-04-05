import { apiRequest, proTableRequest } from "@/api";
import type { CreateUserDto } from "@/api/types/CreateUserDto";
import type { OptionItem } from "@/api/types/OptionItem";
import type { ResetPasswordResp } from "@/api/types/ResetPasswordResp";
import type { UpdateUserPasswordPayload } from "@/api/types/UpdateUserPasswordPayload";
import type { UpdateUserPayload } from "@/api/types/UpdateUserPayload";
import type { UpdateUserStatusPayload } from "@/api/types/UpdateUserStatusPayload";
import type { UserItemResp } from "@/api/types/UserItemResp";
import type { UserStatus } from "@/api/types/UserStatus";

/** ProTable 实际传递给用户列表接口的搜索参数（column dataIndex 与后端字段一一对应） */
export type UserTableParams = {
    current?: number;
    pageSize?: number;
    username?: string;
    realName?: string;
    email?: string;
    status?: UserStatus;
};

/**
 * 用户管理API服务
 */
export const userAPI = {
    getTableData: (params?: UserTableParams) =>
        proTableRequest<UserItemResp, UserTableParams>({
            url: "/api/system/users",
            params,
        }),

    create: (data: CreateUserDto) =>
        apiRequest<UserItemResp, CreateUserDto>({
            url: "/api/system/users",
            method: "POST",
            params: data,
            skipSuccessMsg: true,
        }),

    update: (id: number, data: UpdateUserPayload) =>
        apiRequest<UserItemResp, UpdateUserPayload>({
            url: `/api/system/users/${id}`,
            method: "PUT",
            params: data,
            skipSuccessMsg: true,
        }),

    delete: (id: number) =>
        apiRequest<void>({
            url: `/api/system/users/${id}`,
            method: "DELETE",
            skipSuccessMsg: true,
        }),

    updateStatus: (id: number, data: UpdateUserStatusPayload) =>
        apiRequest<void, UpdateUserStatusPayload>({
            url: `/api/system/users/${id}/status`,
            method: "PUT",
            params: data,
            skipSuccessMsg: true,
        }),

    resetPassword: (id: number) =>
        apiRequest<ResetPasswordResp, UpdateUserPasswordPayload>({
            url: `/api/system/users/${id}/password`,
            method: "PUT",
            body: JSON.stringify({} as UpdateUserPasswordPayload),
            skipSuccessMsg: true,
        }),

    unlock: (id: number) =>
        apiRequest<void>({
            url: `/api/system/users/${id}/unlock`,
            method: "PUT",
            skipSuccessMsg: true,
        }),

    getStatusOptions: () =>
        apiRequest<OptionItem<string>[]>({
            url: "/api/system/users/status-options",
        }),
};
