import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
    queryState: {
        "dashboard/stats": {
            data: {
                totalUsers: 10,
                activeUsers: 8,
                todayLogins: 3,
                systemUptime: "12h",
            },
            isLoading: false,
        },
        "dashboard/health": {
            data: {
                memoryUsed: 1024,
                memoryTotal: 2048,
                cpuUsed: 20,
                cpuTotal: 100,
                diskUsed: 50,
                diskTotal: 100,
            },
            isLoading: false,
        },
        "dashboard/metrics": {
            data: {
                avgResponseTime: 120,
                errorRate: 1.5,
                totalRequests: 999,
            },
            isLoading: false,
        },
        "dashboard/trends": {
            data: {
                dailyLogins: [{ date: "2026-01-01", count: 1 }],
                hourlyActive: [{ date: "10:00", count: 2 }],
            },
            isLoading: false,
        },
    } as Record<string, { data: any; isLoading: boolean }>,
    useApiQuery: vi.fn((key: string) => mocks.queryState[key]),
}));

vi.mock("@tanstack/react-router", () => ({
    createFileRoute: () => () => ({}),
    createRootRoute: () => ({}),
    Navigate: () => null,
    Outlet: () => null,
    redirect: vi.fn(),
}));

vi.mock("@/routeTree.gen", () => ({
    routeTree: {},
}));

vi.mock("@/integrations/tanstack-query/layout", () => ({
    TanStackDevtoolsLayout: () => null,
}));

vi.mock("@ant-design/plots", () => ({
    Line: () => <div>line-chart</div>,
    Column: () => <div>column-chart</div>,
}));

vi.mock("antd", () => ({
    Card: ({
        title,
        extra,
        children,
    }: {
        title?: React.ReactNode;
        extra?: React.ReactNode;
        children?: React.ReactNode;
    }) => (
        <section>
            <div>{title}</div>
            <div>{extra}</div>
            {children}
        </section>
    ),
    Progress: ({
        percent,
        status,
        strokeColor,
    }: {
        percent?: number;
        status?: string;
        strokeColor?: string;
    }) => <div>{`${percent ?? 0}%|${status ?? "normal"}|${strokeColor ?? "none"}`}</div>,
    Statistic: ({ title, value }: { title?: React.ReactNode; value?: React.ReactNode }) => (
        <div>
            <span>{title}</span>
            <span>{value}</span>
        </div>
    ),
    Skeleton: ({ children }: { children?: React.ReactNode }) => <>{children}</>,
    Row: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
    Col: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock("@/integrations/react-query", () => ({
    useApiQuery: mocks.useApiQuery,
}));

vi.mock("@/api/dashboard", () => ({
    dashboardAPI: {
        getStats: vi.fn(),
        getHealth: vi.fn(),
        getMetrics: vi.fn(),
        getTrends: vi.fn(),
    },
}));

import { DashboardPage } from "./index";

beforeEach(() => {
    mocks.queryState["dashboard/stats"] = {
        data: {
            totalUsers: 10,
            activeUsers: 8,
            todayLogins: 3,
            systemUptime: "12h",
        },
        isLoading: false,
    };
    mocks.queryState["dashboard/health"] = {
        data: {
            memoryUsed: 1024,
            memoryTotal: 2048,
            cpuUsed: 20,
            cpuTotal: 100,
            diskUsed: 50,
            diskTotal: 100,
        },
        isLoading: false,
    };
    mocks.queryState["dashboard/metrics"] = {
        data: {
            avgResponseTime: 120,
            errorRate: 1.5,
            totalRequests: 999,
        },
        isLoading: false,
    };
    mocks.queryState["dashboard/trends"] = {
        data: {
            dailyLogins: [{ date: "2026-01-01", count: 1 }],
            hourlyActive: [{ date: "10:00", count: 2 }],
        },
        isLoading: false,
    };
});

describe("DashboardPage", () => {
    it("renders dashboard stats, health, metrics, and trend sections", () => {
        render(<DashboardPage />);

        expect(screen.getByText("总用户数")).toBeInTheDocument();
        expect(screen.getByText("10")).toBeInTheDocument();
        expect(screen.getByText("系统健康状态")).toBeInTheDocument();
        expect(screen.getByText("内存使用量")).toBeInTheDocument();
        expect(screen.getByText("1.0KB / 2.0KB")).toBeInTheDocument();
        expect(screen.getByText("7天性能指标")).toBeInTheDocument();
        expect(screen.getByText("120ms")).toBeInTheDocument();
        expect(screen.getByText("30天用户登录趋势图")).toBeInTheDocument();
        expect(screen.getByText("line-chart")).toBeInTheDocument();
        expect(screen.getByText("column-chart")).toBeInTheDocument();
    });

    it("renders loading skeletons and default zero values", () => {
        mocks.queryState["dashboard/stats"] = { data: undefined, isLoading: true };
        mocks.queryState["dashboard/health"] = { data: undefined, isLoading: true };
        mocks.queryState["dashboard/metrics"] = { data: undefined, isLoading: true };
        mocks.queryState["dashboard/trends"] = { data: undefined, isLoading: true };

        render(<DashboardPage />);

        expect(screen.getByText("总用户数")).toBeInTheDocument();
        expect(screen.getAllByText("0%|normal|none")).toHaveLength(3);
        expect(screen.getAllByText("0 / 0")).toHaveLength(2);
        expect(screen.getByText("0ms")).toBeInTheDocument();
    });

    it("renders warning and exception progress states when thresholds are exceeded", () => {
        mocks.queryState["dashboard/health"] = {
            data: {
                memoryUsed: 70,
                memoryTotal: 100,
                cpuUsed: 85,
                cpuTotal: 100,
                diskUsed: 95,
                diskTotal: 100,
            },
            isLoading: false,
        };

        render(<DashboardPage />);

        expect(screen.getByText("70%|normal|#faad14")).toBeInTheDocument();
        expect(screen.getByText("85%|exception|none")).toBeInTheDocument();
        expect(screen.getByText("95%|exception|none")).toBeInTheDocument();
    });
});
