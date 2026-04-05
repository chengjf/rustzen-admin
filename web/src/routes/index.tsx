import {
    CheckCircleOutlined,
    ClockCircleOutlined,
    ExclamationCircleOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { Column, Line } from "@ant-design/plots";
import { createFileRoute } from "@tanstack/react-router";
import { Card, Progress, Statistic, Skeleton, Row, Col } from "antd";
import { useMemo } from "react";

import { dashboardAPI } from "@/api/dashboard";
import { useApiQuery } from "@/integrations/react-query";
import { calculatePercent, convertUnit } from "@/util";

// =============================================================================
// 1. 路由定义
// =============================================================================

export const Route = createFileRoute("/")({
    component: DashboardPage,
    notFoundComponent: () => (
        <div className="flex h-full items-center justify-center text-xl font-bold">
            404 Not Found
        </div>
    ),
});

// =============================================================================
// 2. 页面主组件
// =============================================================================

export function DashboardPage() {
    return (
        <div className="flex flex-col gap-4 p-1">
            {/* 统计指标行 */}
            <StatsRow />

            {/* 健康状态与性能指标 */}
            <Row gutter={[16, 16]}>
                <Col span={12}>
                    <HealthCard />
                </Col>
                <Col span={12}>
                    <MetricsCard />
                </Col>
            </Row>

            {/* 趋势图表行 */}
            <UserActivityTrendRow />
        </div>
    );
}

// =============================================================================
// 3. 子组件实现
// =============================================================================

/**
 * 顶部统计卡片
 */
const StatsRow = () => {
    const { data: stats, isLoading } = useApiQuery("dashboard/stats", dashboardAPI.getStats);

    const config = useMemo(
        () => [
            {
                title: "总用户数",
                value: stats?.totalUsers,
                prefix: <UserOutlined />,
                color: "#3f8600",
            },
            {
                title: "活跃用户数",
                value: stats?.activeUsers,
                prefix: <TeamOutlined />,
                color: "#1890ff",
            },
            {
                title: "今日登录数",
                value: stats?.todayLogins,
                prefix: <ClockCircleOutlined />,
                color: "#722ed1",
            },
            {
                title: "系统运行时间",
                value: stats?.systemUptime,
                prefix: <CheckCircleOutlined />,
                color: "#52c41a",
            },
        ],
        [stats],
    );

    return (
        <Row gutter={[16, 16]}>
            {config.map((item, index) => (
                <Col key={index} span={6}>
                    <Card bordered={false} hoverable>
                        <Skeleton loading={isLoading} active paragraph={{ rows: 1 }}>
                            <Statistic
                                title={item.title}
                                value={item.value}
                                prefix={item.prefix}
                                valueStyle={{ color: item.color }}
                            />
                        </Skeleton>
                    </Card>
                </Col>
            ))}
        </Row>
    );
};

/**
 * 系统健康状态 (内存/CPU/磁盘)
 */
const HealthCard = () => {
    const { data: health, isLoading } = useApiQuery("dashboard/health", dashboardAPI.getHealth);

    // 防御性计算，防止除以 0 或 undefined
    const memoryUsage = useMemo(
        () => calculatePercent(health?.memoryUsed, health?.memoryTotal),
        [health],
    );
    const cpuUsage = useMemo(() => calculatePercent(health?.cpuUsed, health?.cpuTotal), [health]);
    const diskUsage = useMemo(
        () => calculatePercent(health?.diskUsed, health?.diskTotal),
        [health],
    );

    return (
        <Card
            title="系统健康状态"
            extra={<ExclamationCircleOutlined className="text-gray-400" />}
            bordered={false}
            className="h-full"
        >
            <Skeleton loading={isLoading} active>
                <div className="flex flex-col gap-6">
                    <ProgressItem
                        label="内存使用量"
                        percent={memoryUsage}
                        used={convertUnit(health?.memoryUsed)}
                        total={convertUnit(health?.memoryTotal)}
                        threshold={80}
                    />
                    <ProgressItem
                        label="CPU 使用量"
                        percent={cpuUsage}
                        used={health?.cpuUsed?.toFixed(1)}
                        total={String(health?.cpuTotal ?? 0)}
                        threshold={80}
                    />
                    <ProgressItem
                        label="磁盘使用量"
                        percent={diskUsage}
                        used={convertUnit(health?.diskUsed)}
                        total={convertUnit(health?.diskTotal)}
                        threshold={90}
                    />
                </div>
            </Skeleton>
        </Card>
    );
};

/**
 * 7天性能指标
 */
const MetricsCard = () => {
    const { data: metrics, isLoading } = useApiQuery("dashboard/metrics", dashboardAPI.getMetrics);

    return (
        <Card
            title="7天性能指标"
            bordered={false}
            className="h-full flex flex-col"
            styles={{ body: { flex: 1, display: "flex", alignItems: "center" } }}
        >
            <Skeleton loading={isLoading} active>
                <Row gutter={16} className="w-full">
                    <MetricItem
                        value={`${metrics?.avgResponseTime ?? 0}ms`}
                        label="平均响应时间"
                        color="text-blue-600"
                    />
                    <MetricItem
                        value={`${metrics?.errorRate?.toFixed(1) ?? 0}%`}
                        label="错误率"
                        color="text-red-500"
                    />
                    <MetricItem
                        value={`${metrics?.totalRequests ?? 0} 次`}
                        label="总请求次数"
                        color="text-purple-600"
                    />
                </Row>
            </Skeleton>
        </Card>
    );
};

/**
 * 趋势图表
 */
const UserActivityTrendRow = () => {
    const { data, isLoading } = useApiQuery("dashboard/trends", dashboardAPI.getTrends);

    return (
        <Row gutter={[16, 16]}>
            <Col span={12}>
                <Card title="30天用户登录趋势图" bordered={false}>
                    <Skeleton loading={isLoading} active>
                        <Line
                            data={data?.dailyLogins || []}
                            xField="date"
                            yField="count"
                            height={300}
                            autoFit
                            axis={{ y: { labelFormatter: (v: number) => Math.round(v) } }}
                            smooth
                        />
                    </Skeleton>
                </Card>
            </Col>
            <Col span={12}>
                <Card title="24小时活跃用户趋势图" bordered={false}>
                    <Skeleton loading={isLoading} active>
                        <Column
                            data={data?.hourlyActive || []}
                            xField="date"
                            yField="count"
                            height={300}
                            autoFit
                            axis={{ y: { labelFormatter: (v: number) => Math.round(v) } }}
                            style={{ radiusTopLeft: 4, radiusTopRight: 4 }}
                        />
                    </Skeleton>
                </Card>
            </Col>
        </Row>
    );
};

// =============================================================================
// 4. 私有原子组件 (Private Sub-components)
// =============================================================================

const ProgressItem = ({ label, percent, used, total, threshold }: any) => (
    <div>
        <div className="mb-2 flex justify-between text-sm">
            <span className="text-gray-500">{label}</span>
            <span className="font-medium">
                {used} / {total}
            </span>
        </div>
        <Progress
            percent={percent}
            status={percent > threshold ? "exception" : "normal"}
            strokeColor={percent > threshold ? undefined : percent > 60 ? "#faad14" : undefined}
        />
    </div>
);

const MetricItem = ({ value, label, color }: any) => (
    <Col span={8} className="text-center">
        <div className={`text-2xl font-bold ${color}`}>{value}</div>
        <div className="text-gray-500 text-xs mt-1 uppercase tracking-wider">{label}</div>
    </Col>
);
