import { apiRequest } from "@/api";
import type {
    StatsResp,
    SystemInfo,
    SystemMetricsDataResp,
    UserTrendsResp,
} from "@/api/types";

export const dashboardAPI = {
    getStats: () => apiRequest<StatsResp>({ url: "/api/dashboard/stats" }),
    getHealth: () => apiRequest<SystemInfo>({ url: "/api/dashboard/health" }),
    getMetrics: () => apiRequest<SystemMetricsDataResp>({ url: "/api/dashboard/metrics" }),
    getTrends: () => apiRequest<UserTrendsResp>({ url: "/api/dashboard/trends" }),
};
