import { apiRequest, proTableRequest } from "@/api";
import type {
    RoleItemResp,
    RoleQuery,
    CreateRoleDto,
    UpdateRolePayload,
    OptionItem,
} from "@/api/types";

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
        }),

    update: (id: number, data: UpdateRolePayload) =>
        apiRequest<RoleItemResp, UpdateRolePayload>({
            url: `/api/system/roles/${id}`,
            method: "PUT",
            params: data,
        }),

    delete: (id: number) => apiRequest<void>({ url: `/api/system/roles/${id}`, method: "DELETE" }),

    getOptions: () => apiRequest<OptionItem<number>[]>({ url: "/api/system/roles/options" }),
};
