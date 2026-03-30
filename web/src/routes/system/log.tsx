import { type ActionType, type ProColumns, ProTable } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Segmented, Tag, message } from "antd";
// 修复 TS6133: 移除未使用的 React 默认导入，仅保留需要的 Hooks
import { useRef, useCallback, useMemo, useState } from "react";

import { logAPI } from "@/api/system/log";
import type { LogItemResp } from "@/api/types/LogItemResp";
import { AuthWrap } from "@/components/auth";
import { useLocalStore } from "@/stores/useLocalStore";

// =============================================================================
// 1. 路由定义 (Router Definition)
// =============================================================================

export const Route = createFileRoute("/system/log")({
    component: LogPage,
});

// =============================================================================
// 2. 常量配置 (Constants)
// =============================================================================

const ACTION_OPTIONS = [
    { label: "全部", value: "" },
    { label: "登录", value: "AUTH_LOGIN" },
    { label: "GET", value: "HTTP_GET" },
    { label: "POST", value: "HTTP_POST" },
    { label: "PUT", value: "HTTP_PUT" },
    { label: "DELETE", value: "HTTP_DELETE" },
];

const ACTION_COLOR_MAP: Record<string, string> = {
    HTTP_GET: "default",
    HTTP_POST: "processing",
    HTTP_PUT: "warning",
    HTTP_DELETE: "error",
    AUTH_LOGIN: "success",
};

// =============================================================================
// 3. 页面主组件 (Main Component)
// =============================================================================

function LogPage() {
    const actionRef = useRef<ActionType>(null);
    const [exportLoading, setExportLoading] = useState(false);

    const [storedAction, setActionType] = useLocalStore("log-action");
    const actionType = useMemo(() => storedAction ?? "", [storedAction]);

    /**
     * 导出逻辑处理
     */
    const handleExport = useCallback(async () => {
        setExportLoading(true);
        try {
            const filename = await logAPI.exportLogList();
            message.success(`日志已导出为 ${filename}`);
        } catch (error) {
            console.error("[Log Export Error]:", error);
            // Error toast is already shown by handleError, no need to show again
        } finally {
            setExportLoading(false);
        }
    }, []);

    const columns: ProColumns<LogItemResp>[] = useMemo(
        () => [
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
                render: (_, record) =>
                    record.username || <span className="text-gray-400">匿名用户</span>,
            },
            {
                title: "操作类型",
                align: "center",
                dataIndex: "action",
                width: 150,
                render: (_, record) => (
                    <Tag color={ACTION_COLOR_MAP[record.action] || "default"} variant="outlined">
                        {record.action}
                    </Tag>
                ),
            },
            {
                title: "操作描述",
                align: "left",
                dataIndex: "description",
                ellipsis: true,
            },
            {
                title: "状态",
                align: "center",
                dataIndex: "status",
                width: 100,
                render: (_, record) => (
                    <Tag color={record.status === "SUCCESS" ? "success" : "error"} variant="solid">
                        {record.status}
                    </Tag>
                ),
            },
            {
                title: "IP地址",
                align: "center",
                dataIndex: "ipAddress",
                width: 140,
                render: (_, record) => record.ipAddress || "-",
            },
            {
                title: "耗时",
                align: "right",
                dataIndex: "durationMs",
                width: 100,
                render: (_, record) => {
                    if (!record.durationMs) return "-";
                    const isSlow = record.durationMs > 1000;
                    return (
                        <span className={isSlow ? "text-red-500 font-medium" : ""}>
                            {record.durationMs}ms
                        </span>
                    );
                },
            },
            {
                title: "操作时间",
                align: "center",
                dataIndex: "createdAt",
                width: 180,
                valueType: "dateTime",
                sorter: true,
            },
        ],
        [],
    );

    const toolBarRender = useCallback(
        () => [
            <AuthWrap code="system:log:export" key="export-wrap">
                <Button key="export" type="primary" loading={exportLoading} onClick={handleExport}>
                    导出日志
                </Button>
            </AuthWrap>,
        ],
        [exportLoading, handleExport],
    );

    return (
        <AuthWrap code="system:log:list">
            <ProTable<LogItemResp>
                headerTitle={
                    <Segmented
                        value={actionType}
                        options={ACTION_OPTIONS}
                        onChange={(val) => setActionType(val as string)}
                    />
                }
                actionRef={actionRef}
                rowKey="id"
                scroll={{ y: "calc(100vh - 300px)" }}
                columns={columns}
                params={{ action: actionType }}
                request={async (params, sorter) => {
                    // 过滤无效排序值
                    const validSorter = Object.fromEntries(
                        Object.entries(sorter).filter(([, v]) => v !== null),
                    );
                    return await logAPI.getTableData({
                        ...params,
                        ...validSorter,
                    });
                }}
                toolBarRender={toolBarRender}
                pagination={{
                    defaultPageSize: 20,
                    showSizeChanger: true,
                }}
                search={false}
            />
        </AuthWrap>
    );
}
