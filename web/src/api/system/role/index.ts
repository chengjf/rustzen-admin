import { apiRequest, proTableRequest } from "@/api";
import type { CreateRoleDto } from "@/api/types/CreateRoleDto";
import type { OptionItem } from "@/api/types/OptionItem";
import type { RoleItemResp } from "@/api/types/RoleItemResp";
import type { RoleQuery } from "@/api/types/RoleQuery";
import type { UpdateRolePayload } from "@/api/types/UpdateRolePayload";

/**
 * 角色管理API服务
 */
export const roleAPI = {
    getTableData: (params?: Partial<RoleQuery>) =>
        proTableRequest<RoleItemResp, Partial<RoleQuery>>({
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
