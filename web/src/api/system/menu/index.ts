import { apiRequest, proTableRequest } from "@/api";

/**
 * 菜单管理API服务
 */
export const menuAPI = {
    getTableData: (params?: Menu.QueryParams) => {
        return apiRequest<Menu.Item[], Menu.QueryParams>({
            url: "/api/system/menus",
            params,
        });
    },

    create: (data: Menu.CreateAndUpdateRequest) =>
        apiRequest<Menu.Item, Menu.CreateAndUpdateRequest>({
            url: "/api/system/menus",
            method: "POST",
            params: data,
        }),

    update: (id: number, data: Menu.CreateAndUpdateRequest) =>
        apiRequest<Menu.Item, Menu.CreateAndUpdateRequest>({
            url: `/api/system/menus/${id}`,
            method: "PUT",
            params: data,
        }),

    delete: (id: number) => apiRequest<void>({ url: `/api/system/menus/${id}`, method: "DELETE" }),

    getOptions: () =>
        apiRequest<Api.OptionItem[]>({ url: "/api/system/menus/options" }).then((res) => [
            { label: "Root", value: 0 },
            ...res,
        ]),
    getOptionsWithCode: (params?: Menu.OptionsWithCodeQuery) =>
        apiRequest<Api.MenuTreeOption[], Menu.OptionsWithCodeQuery>({
            url: "/api/system/menus/options-with-code",
            params,
        }),
};
