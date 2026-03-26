import { apiRequest } from "@/api";
import type {
    MenuItemResp,
    MenuQuery,
    CreateMenuDto,
    UpdateMenuPayload,
    MenuType,
    OptionItem,
    OptionsWithCodeQuery,
    MenuTreeOption,
} from "@/api/types";

/**
 * 菜单管理API服务
 */
export const menuAPI = {
    getTableData: (params?: Partial<MenuQuery>) => {
        return apiRequest<MenuItemResp[], Partial<MenuQuery>>({
            url: "/api/system/menus",
            params,
        });
    },

    create: (data: CreateMenuDto) =>
        apiRequest<MenuItemResp, CreateMenuDto>({
            url: "/api/system/menus",
            method: "POST",
            params: data,
        }),

    update: (id: number, data: UpdateMenuPayload) =>
        apiRequest<MenuItemResp, UpdateMenuPayload>({
            url: `/api/system/menus/${id}`,
            method: "PUT",
            params: data,
        }),

    delete: (id: number) => apiRequest<void>({ url: `/api/system/menus/${id}`, method: "DELETE" }),

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
