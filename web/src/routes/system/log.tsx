import type { ProColumns } from "@ant-design/pro-components";
import { ProTable } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Segmented, Tag } from "antd";

import { logAPI } from "@/api/system/log";
import type { LogItemResp } from "@/api/types/LogItemResp";
import { AuthWrap } from "@/components/auth";
import { useLocalStore } from "@/stores/useLocalStore";

export const Route = createFileRoute("/system/log")({
    component: LogPage,
});
const actionOptions = [
    { label: "全部", value: "" },
    { label: "登录", value: "AUTH_LOGIN" },
    { label: "GET", value: "HTTP_GET" },
    { label: "POST", value: "HTTP_POST" },
    { label: "PUT", value: "HTTP_PUT" },
    { label: "DELETE", value: "HTTP_DELETE" },
];
function LogPage() {
    const [actionType, setActionType] = useLocalStore("log-action");
    return (
        <AuthWrap code="system:log:list">
            <ProTable<LogItemResp>
                rowKey="id"
                scroll={{ y: "calc(100vh - 383px)" }}
                columns={columns}
                params={{ action: actionType }}
                request={logAPI.getTableData}
                headerTitle={
                    <Segmented
                        value={actionType}
                        options={actionOptions}
                        onChange={(val) => {
                            setActionType(val);
                        }}
                    />
                }
                toolBarRender={() => [
                    <AuthWrap code="system:log:export">
                        <Button
                            key="export"
                            type="primary"
                            onClick={() => {
                                logAPI.exportLogList();
                            }}
                        >
                            导出
                        </Button>
                    </AuthWrap>,
                ]}
            />
        </AuthWrap>
    );
}

const actionColorMap: Record<string, string> = {
    HTTP_GET: "default",
    HTTP_POST: "processing",
    HTTP_PUT: "warning",
    HTTP_DELETE: "error",
    AUTH_LOGIN: "success",
};
const columns: ProColumns<LogItemResp>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 80,
        hideInSearch: true,
        align: "center",
    },
    {
        title: "用户",
        align: "center",
        dataIndex: "username",
        width: 120,
        render: (_, record) => record.username || "匿名用户",
    },
    {
        title: "操作",
        align: "center",
        dataIndex: "action",
        width: 150,
        hideInSearch: true,
        render: (_, record) => {
            const action = record.action;
            const color = actionColorMap[action];
            return (
                <Tag color={color} variant="outlined">
                    {action}
                </Tag>
            );
        },
    },
    {
        title: "描述",
        align: "center",
        dataIndex: "description",
        ellipsis: true,
    },
    {
        title: "状态",
        align: "center",
        dataIndex: "status",
        width: 100,
        hideInSearch: true,
        render: (_, record) => {
            const status = record.status;
            const color = status === "SUCCESS" ? "success" : "error";
            return (
                <Tag color={color} variant="solid">
                    {status}
                </Tag>
            );
        },
    },
    {
        title: "IP地址",
        align: "center",
        dataIndex: "ipAddress",
        width: 120,
        render: (_, record) => record.ipAddress || "-",
    },
    {
        title: "耗时",
        align: "center",
        dataIndex: "durationMs",
        width: 80,
        hideInSearch: true,
        render: (_, record) => {
            if (!record.durationMs) return "-";
            return `${record.durationMs}ms`;
        },
    },
    {
        title: "创建时间",
        align: "center",
        dataIndex: "createdAt",
        width: 200,
        valueType: "dateTime",
        hideInSearch: true,
    },
];
