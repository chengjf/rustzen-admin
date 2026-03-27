import { apiRequest, proTableRequest } from "@/api";
import type { CreateDictDto } from "@/api/types/CreateDictDto";
import type { DictItemResp } from "@/api/types/DictItemResp";
import type { DictQuery } from "@/api/types/DictQuery";
import type { OptionItem } from "@/api/types/OptionItem";
import type { UpdateDictPayload } from "@/api/types/UpdateDictPayload";

/**
 * 字典管理API服务
 */
export const dictAPI = {
    getTableData: (params?: Partial<DictQuery>) =>
        proTableRequest<DictItemResp, Partial<DictQuery>>({
            url: "/api/system/dicts",
            params,
        }),

    create: (data: CreateDictDto) =>
        apiRequest<DictItemResp, CreateDictDto>({
            url: "/api/system/dicts",
            method: "POST",
            params: data,
        }),

    update: (id: number, data: UpdateDictPayload) =>
        apiRequest<DictItemResp, UpdateDictPayload>({
            url: `/api/system/dicts/${id}`,
            method: "PUT",
            params: data,
        }),

    delete: (id: number) =>
        apiRequest<void>({
            url: `/api/system/dicts/${id}`,
            method: "DELETE",
        }),

    getOptions: () => apiRequest<OptionItem<string>[]>({ url: "/api/system/dicts/options" }),

    getOptionsByType: (type: string) =>
        apiRequest<DictItemResp[]>({ url: `/api/system/dicts/type/${type}` }),
};
