import {
    CheckCircleOutlined,
    ClockCircleOutlined,
    ExclamationCircleOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { Column, Line } from "@ant-design/plots";
import { createFileRoute } from "@tanstack/react-router";
import { Card, Progress, Statistic } from "antd";

import { dashboardAPI } from "@/api/dashboard";
import { useApiQuery } from "@/integrations/react-query";
import { calculatePercent, convertUnit } from "@/util";

export const Route = createFileRoute("/")({
    component: DashboardPage,
    notFoundComponent: () => <div>404 Not Found 1111</div>,
});

function DashboardPage() {
    return (
        <div className="flex flex-col gap-4">
            {/* 统计卡片 */}
            <StatsCard />

            {/* 系统健康状态 */}
            <div className="grid grid-cols-2 gap-4">
                <HealthCard />
                <MetricsCard />
            </div>

            {/* 用户活动趋势 */}
            <UserActivityTrendCard />
        </div>
    );
}

// 统计卡片
const StatsCard = () => {
    const { data: stats } = useApiQuery("dashboard/stats", dashboardAPI.getStats);
    return (
        <div className="grid grid-cols-4 gap-4">
            <Card>
                <Statistic
                    title="总用户数"
                    value={stats?.totalUsers}
                    prefix={<UserOutlined />}
                    valueStyle={{ color: "#3f8600" }}
                />
            </Card>
            <Card>
                <Statistic
                    title="活跃用户数"
                    value={stats?.activeUsers}
                    prefix={<TeamOutlined />}
                    valueStyle={{ color: "#1890ff" }}
                />
            </Card>
            <Card>
                <Statistic
                    title="今日登录数"
                    value={stats?.todayLogins}
                    prefix={<ClockCircleOutlined />}
                    valueStyle={{ color: "#722ed1" }}
                />
            </Card>

            <Card>
                <Statistic
                    title="系统运行时间"
                    value={stats?.systemUptime}
                    prefix={<CheckCircleOutlined />}
                    valueStyle={{ color: "#52c41a" }}
                />
            </Card>
        </div>
    );
};

// 系统健康状态
const HealthCard = () => {
    const { data: health } = useApiQuery("dashboard/health", dashboardAPI.getHealth);
    const memoryUsage = calculatePercent(health?.memoryUsed, health?.memoryTotal);
    const cpuUsage = calculatePercent(health?.cpuUsed, health?.cpuTotal);
    const diskUsage = calculatePercent(health?.diskUsed, health?.diskTotal);

    return (
        <Card title="系统健康状态" extra={<ExclamationCircleOutlined />}>
            <div className="flex flex-col gap-5">
                <div>
                    <div className="mb-2 flex justify-between">
                        <span>内存使用量</span>
                        <span>
                            {convertUnit(health?.memoryUsed)} / {convertUnit(health?.memoryTotal)}
                        </span>
                    </div>
                    <Progress
                        percent={memoryUsage}
                        status={memoryUsage > 80 ? "exception" : "normal"}
                    />
                </div>
                <div>
                    <div className="mb-2 flex justify-between">
                        <span>CPU 使用量</span>
                        <span>
                            {health?.cpuUsed.toFixed(1)} / {health?.cpuTotal}
                        </span>
                    </div>
                    <Progress percent={cpuUsage} status={cpuUsage > 80 ? "exception" : "normal"} />
                </div>
                <div>
                    <div className="mb-2 flex justify-between">
                        <span>磁盘使用量</span>
                        <span>
                            {convertUnit(health?.diskUsed)} / {convertUnit(health?.diskTotal)}
                        </span>
                    </div>
                    <Progress
                        percent={diskUsage}
                        status={diskUsage > 90 ? "exception" : "normal"}
                    />
                </div>
            </div>
        </Card>
    );
};

// 性能指标
const MetricsCard = () => {
    const { data: metrics } = useApiQuery("dashboard/metrics", dashboardAPI.getMetrics);
    return (
        <Card
            title="7天性能指标"
            rootClassName="flex flex-col"
            classNames={{
                body: "flex-1 place-content-center",
            }}
        >
            <div className="grid grid-cols-3 gap-4">
                <div className="text-center">
                    <div className="text-2xl font-bold text-blue-600">
                        {metrics?.avgResponseTime}ms
                    </div>
                    <div className="text-gray-500">平均响应时间</div>
                </div>
                <div className="text-center">
                    <div className="text-2xl font-bold text-green-600">
                        {metrics?.errorRate?.toFixed(1)}%
                    </div>
                    <div className="text-gray-500">错误率</div>
                </div>
                <div className="text-center">
                    <div className="text-2xl font-bold text-purple-600">
                        {metrics?.totalRequests} 次请求
                    </div>
                    <div className="text-gray-500">总请求次数</div>
                </div>
            </div>
        </Card>
    );
};

// 用户活动趋势
const UserActivityTrendCard = () => {
    const { data } = useApiQuery("dashboard/trends", dashboardAPI.getTrends);
    return (
        <div className="grid grid-cols-2 gap-4">
            <Card title="30天用户登录趋势图">
                <Line
                    data={data?.dailyLogins || []}
                    xField="date"
                    yField="count"
                    height={300}
                    axis={{
                        y: {
                            labelFormatter: (v: number) => Math.round(v),
                        },
                    }}
                />
            </Card>
            <Card title="24小时活跃用户趋势图">
                <Column
                    data={data?.hourlyActive || []}
                    xField="date"
                    yField="count"
                    height={300}
                    axis={{
                        y: {
                            labelFormatter: (v: number) => Math.round(v),
                        },
                    }}
                    style={{
                        radiusTopLeft: 10,
                        radiusTopRight: 10,
                    }}
                />
            </Card>
        </div>
    );
};
