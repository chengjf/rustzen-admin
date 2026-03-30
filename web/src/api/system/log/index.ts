import { apiDownload, proTableRequest } from "@/api";
import type { LogItemResp } from "@/api/types/LogItemResp";
import type { LogQuery } from "@/api/types/LogQuery";

/**
 * 日志管理API服务
 */
export const logAPI = {
    getTableData: (params?: Partial<LogQuery>) =>
        proTableRequest<LogItemResp, Partial<LogQuery>>({
            url: "/api/system/logs",
            params,
        }),

    exportLogList: () => {
        return apiDownload({ url: "/api/system/logs/export" });
    },
};
