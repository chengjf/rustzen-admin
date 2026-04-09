import {
    CheckCircleOutlined,
    ClockCircleOutlined,
    ExclamationCircleOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { Card, Col, Progress, Row, Skeleton, Statistic } from "antd";
import { lazy, Suspense, useMemo } from "react";

import { dashboardAPI } from "@/api/dashboard";
import { useApiQuery } from "@/integrations/react-query";
import { calculatePercent, convertUnit } from "@/util";

export function DashboardPage() {
    return (
        <div className="flex flex-col gap-4 p-1">
            <StatsRow />
            <Row gutter={[16, 16]}>
                <Col span={12}>
                    <HealthCard />
                </Col>
                <Col span={12}>
                    <MetricsCard />
                </Col>
            </Row>
            <UserActivityTrendRow />
        </div>
    );
}

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

const HealthCard = () => {
    const { data: health, isLoading } = useApiQuery("dashboard/health", dashboardAPI.getHealth);
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

const UserActivityTrendRow = () => {
    const { data, isLoading } = useApiQuery("dashboard/trends", dashboardAPI.getTrends);

    return (
        <Suspense
            fallback={
                <Row gutter={[16, 16]}>
                    <Col span={12}>
                        <Card title="30天用户登录趋势图" bordered={false}>
                            <Skeleton active paragraph={{ rows: 12 }} />
                        </Card>
                    </Col>
                    <Col span={12}>
                        <Card title="24小时活跃用户趋势图" bordered={false}>
                            <Skeleton active paragraph={{ rows: 12 }} />
                        </Card>
                    </Col>
                </Row>
            }
        >
            <LazyDashboardCharts data={data} isLoading={isLoading} />
        </Suspense>
    );
};

const LazyDashboardCharts = lazy(() =>
    import("@/components/dashboard/DashboardCharts").then((module) => ({
        default: module.DashboardCharts,
    })),
);

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
