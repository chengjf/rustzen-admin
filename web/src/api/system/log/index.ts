import { apiDownload, proTableRequest } from "@/api";
import type { LogItemResp, LogQuery } from "@/api/types";

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
        void apiDownload({ url: "/api/system/logs/export" }).then(async (res) => {
            console.log("downloadName", res);
        });
    },
};
