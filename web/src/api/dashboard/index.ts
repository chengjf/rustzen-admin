import { apiRequest } from "@/api";
import type { StatsResp } from "@/api/types/StatsResp";
import type { SystemInfo } from "@/api/types/SystemInfo";
import type { SystemMetricsDataResp } from "@/api/types/SystemMetricsDataResp";
import type { UserTrendsResp } from "@/api/types/UserTrendsResp";

export const dashboardAPI = {
    getStats: () => apiRequest<StatsResp>({ url: "/api/dashboard/stats" }),
    getHealth: () => apiRequest<SystemInfo>({ url: "/api/dashboard/health" }),
    getMetrics: () => apiRequest<SystemMetricsDataResp>({ url: "/api/dashboard/metrics" }),
    getTrends: () => apiRequest<UserTrendsResp>({ url: "/api/dashboard/trends" }),
};
