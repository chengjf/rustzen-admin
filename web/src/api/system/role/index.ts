import { apiRequest, proTableRequest } from "@/api";
import type { CreateRoleDto } from "@/api/types/CreateRoleDto";
import type { OptionItem } from "@/api/types/OptionItem";
import type { RoleItemResp } from "@/api/types/RoleItemResp";
import type { UpdateRolePayload } from "@/api/types/UpdateRolePayload";

/** ProTable 实际传递给角色列表接口的搜索参数（column dataIndex 与后端字段一一对应） */
export type RoleTableParams = {
    current?: number;
    pageSize?: number;
    name?: string;
    code?: string;
    status?: string;
};

/**
 * 角色管理API服务
 */
export const roleAPI = {
    getTableData: (params?: RoleTableParams) =>
        proTableRequest<RoleItemResp, RoleTableParams>({
            url: "/api/system/roles",
            params,
        }),

    create: (data: CreateRoleDto) =>
        apiRequest<RoleItemResp, CreateRoleDto>({
            url: "/api/system/roles",
            method: "POST",
            params: data,
            skipSuccessMsg: true, // Let page handle success message
        }),

    update: (id: number, data: UpdateRolePayload) =>
        apiRequest<RoleItemResp, UpdateRolePayload>({
            url: `/api/system/roles/${id}`,
            method: "PUT",
            params: data,
            skipSuccessMsg: true, // Let page handle success message
        }),

    delete: (id: number) =>
        apiRequest<void>({
            url: `/api/system/roles/${id}`,
            method: "DELETE",
            skipSuccessMsg: true, // Let page handle success message
        }),

    getOptions: () => apiRequest<OptionItem<number>[]>({ url: "/api/system/roles/options" }),
};
