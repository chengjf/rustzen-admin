import { apiDownload, proTableRequest } from "@/api";
import type { LogItemResp } from "@/api/types/LogItemResp";

/** ProTable 实际传递给日志列表接口的参数（含分页、操作类型快速筛选及排序） */
export type LogTableParams = {
    current?: number;
    pageSize?: number;
    action?: string;
    username?: string;
    description?: string;
    ipAddress?: string;
};

/**
 * 日志管理API服务
 */
export const logAPI = {
    getTableData: (params?: LogTableParams) =>
        proTableRequest<LogItemResp, LogTableParams>({
            url: "/api/system/logs",
            params,
        }),

    exportLogList: () => {
        return apiDownload({ url: "/api/system/logs/export" });
    },
};
