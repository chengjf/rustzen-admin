import { Column, Line } from "@ant-design/plots";
import { Card, Col, Row, Skeleton } from "antd";

import type { UserTrendsResp } from "@/api/types/UserTrendsResp";

interface DashboardChartsProps {
    data?: UserTrendsResp;
    isLoading: boolean;
}

export function DashboardCharts({ data, isLoading }: DashboardChartsProps) {
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
}
