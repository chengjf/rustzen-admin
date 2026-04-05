import { apiRequest } from "@/api";
import type { CreateMenuDto } from "@/api/types/CreateMenuDto";
import type { MenuItemResp } from "@/api/types/MenuItemResp";
import type { MenuTreeOption } from "@/api/types/MenuTreeOption";
import type { MenuType } from "@/api/types/MenuType";
import type { OptionItem } from "@/api/types/OptionItem";
import type { OptionsWithCodeQuery } from "@/api/types/OptionsWithCodeQuery";
import type { UpdateMenuPayload } from "@/api/types/UpdateMenuPayload";

/** ProTable 实际传递给菜单列表接口的搜索参数（column dataIndex 与后端字段一一对应） */
export type MenuTableParams = {
    name?: string;
    code?: string;
    status?: string;
    menuType?: number;
};

/**
 * 菜单管理API服务
 */
export const menuAPI = {
    getTableData: (params?: MenuTableParams) => {
        return apiRequest<MenuItemResp[], MenuTableParams>({
            url: "/api/system/menus",
            params,
        });
    },

    create: (data: CreateMenuDto) =>
        apiRequest<MenuItemResp, CreateMenuDto>({
            url: "/api/system/menus",
            method: "POST",
            params: data,
            skipSuccessMsg: true,
        }),

    update: (id: number, data: UpdateMenuPayload) =>
        apiRequest<MenuItemResp, UpdateMenuPayload>({
            url: `/api/system/menus/${id}`,
            method: "PUT",
            params: data,
            skipSuccessMsg: true,
        }),

    delete: (id: number) =>
        apiRequest<void>({
            url: `/api/system/menus/${id}`,
            method: "DELETE",
            skipSuccessMsg: true,
        }),

    getOptions: () =>
        apiRequest<OptionItem<number>[]>({ url: "/api/system/menus/options" }).then((res) => [
            { label: "Root", value: 0 },
            ...res,
        ]),

    getOptionsWithCode: (params?: OptionsWithCodeQuery) =>
        apiRequest<MenuTreeOption[], OptionsWithCodeQuery>({
            url: "/api/system/menus/options-with-code",
            params,
        }),
};

export type { MenuType };
