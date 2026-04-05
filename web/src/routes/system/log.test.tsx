import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "@/stores/useAuthStore";

const mocks = vi.hoisted(() => ({
    controlledExport: null as null | Promise<string>,
    exportLogList: vi.fn(),
    getTableData: vi.fn(),
    rows: [
        {
            id: 1,
            username: "",
            action: "HTTP_DELETE",
            status: "FAILED",
            ipAddress: "",
            durationMs: 1500,
        },
        {
            id: 2,
            username: "alice",
            action: "AUTH_LOGIN",
            status: "SUCCESS",
            ipAddress: "127.0.0.1",
            durationMs: 0,
        },
        {
            id: 3,
            username: "bob",
            action: "CUSTOM_ACTION",
            status: "SUCCESS",
            ipAddress: "10.0.0.1",
            durationMs: 800,
        },
    ],
    success: vi.fn(),
    setActionType: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
    createFileRoute: () => () => ({}),
}));

vi.mock("@ant-design/pro-components", () => ({
    ProTable: ({
        headerTitle,
        toolBarRender,
        columns,
        request,
        params,
    }: {
        headerTitle?: React.ReactNode;
        toolBarRender?: () => React.ReactNode[];
        columns?: Array<{
            dataIndex?: string;
            render?: (_: unknown, record: any) => React.ReactNode;
        }>;
        request?: (params: any, sorter: Record<string, any>) => Promise<unknown>;
        params?: Record<string, unknown>;
    }) => (
        <div>
            <div>{headerTitle}</div>
            <div>{toolBarRender?.()}</div>
            <div>
                {columns?.flatMap((column, columnIndex) =>
                    mocks.rows.map((row) => (
                        <div key={`${String(column.dataIndex)}-${columnIndex}-${row.id}`}>
                            {column.render?.(row[column.dataIndex as keyof typeof row], row)}
                        </div>
                    )),
                )}
            </div>
            <button
                onClick={() => {
                    void request?.(params || {}, { createdAt: "descend", ignored: null });
                }}
            >
                trigger-request
            </button>
            <div>log-table</div>
        </div>
    ),
}));

vi.mock("antd", () => ({
    Button: ({
        children,
        loading,
        onClick,
    }: {
        children: React.ReactNode;
        loading?: boolean;
        onClick?: () => void;
    }) => <button onClick={onClick}>{loading ? "loading" : children}</button>,
    Segmented: ({
        value,
        options,
        onChange,
    }: {
        value?: string;
        options?: Array<{ label: string; value: string }>;
        onChange?: (value: string) => void;
    }) => (
        <div>
            <span>{value}</span>
            {options?.map((option) => <span key={option.value}>{option.label}</span>)}
            <button onClick={() => onChange?.("AUTH_LOGIN")}>change-action</button>
        </div>
    ),
    Tag: ({ children }: { children: React.ReactNode }) => <span>{children}</span>,
}));

vi.mock("@/api", () => ({
    appMessage: { success: mocks.success },
}));

vi.mock("@/api/system/log", () => ({
    logAPI: {
        exportLogList: mocks.exportLogList,
        getTableData: mocks.getTableData,
    },
}));

vi.mock("@/stores/useLocalStore", () => ({
    useLocalStore: () => ["", mocks.setActionType],
}));

import { LogPage } from "./log";

beforeEach(() => {
    act(() => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, permissions: ["system:log:list", "system:log:export"] },
        });
    });
    mocks.exportLogList.mockResolvedValue("logs.csv");
    mocks.getTableData.mockResolvedValue({ data: [], total: 0, success: true });
});

afterEach(() => {
    act(() => {
        useAuthStore.setState({ token: null, userInfo: null });
    });
    vi.clearAllMocks();
});

describe("LogPage", () => {
    it("renders the log page with action filters and export button", () => {
        render(<LogPage />);

        expect(screen.getByText("log-table")).toBeInTheDocument();
        expect(screen.getByText("全部")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "导出日志" })).toBeInTheDocument();
    });

    it("hides the export button without export permission", () => {
        act(() => {
            useAuthStore.setState({
                token: "token",
                userInfo: { ...mockUserInfo, permissions: ["system:log:list"] },
            });
        });

        render(<LogPage />);

        expect(screen.getByText("log-table")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "导出日志" })).not.toBeInTheDocument();
    });

    it("exports logs and shows a success message", async () => {
        render(<LogPage />);

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "导出日志" }));
        });

        expect(mocks.exportLogList).toHaveBeenCalledTimes(1);
        expect(mocks.success).toHaveBeenCalledWith("日志已导出为 logs.csv");
    });

    it("keeps export failures silent at the toast layer", async () => {
        vi.spyOn(console, "error").mockImplementation(() => {});
        mocks.exportLogList.mockRejectedValueOnce(new Error("boom"));

        render(<LogPage />);

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "导出日志" }));
        });

        expect(mocks.success).not.toHaveBeenCalled();
        expect(console.error).toHaveBeenCalled();
    });

    it("shows loading state while exporting logs", async () => {
        let resolveExport!: (value: string) => void;
        mocks.exportLogList.mockImplementationOnce(
            () =>
                new Promise<string>((resolve) => {
                    resolveExport = resolve;
                }),
        );

        render(<LogPage />);

        fireEvent.click(screen.getByRole("button", { name: "导出日志" }));

        expect(screen.getByRole("button", { name: "loading" })).toBeInTheDocument();

        await act(async () => {
            resolveExport("async.csv");
            await Promise.resolve();
        });

        expect(mocks.success).toHaveBeenCalledWith("日志已导出为 async.csv");
    });

    it("updates the local action filter and passes valid sorter params", async () => {
        render(<LogPage />);

        fireEvent.click(screen.getByRole("button", { name: "change-action" }));
        fireEvent.click(screen.getByRole("button", { name: "trigger-request" }));

        await waitFor(() => {
            expect(mocks.setActionType).toHaveBeenCalledWith("AUTH_LOGIN");
        });
        expect(mocks.getTableData).toHaveBeenCalledWith({
            action: "",
            createdAt: "descend",
        });
    });

    it("renders fallback cells for anonymous users, tags, ip and duration", () => {
        render(<LogPage />);

        expect(screen.getByText("匿名用户")).toBeInTheDocument();
        expect(screen.getByText("HTTP_DELETE")).toBeInTheDocument();
        expect(screen.getByText("CUSTOM_ACTION")).toBeInTheDocument();
        expect(screen.getByText("FAILED")).toBeInTheDocument();
        expect(screen.getAllByText("-").length).toBeGreaterThan(0);
        expect(screen.getByText("1500ms")).toBeInTheDocument();
        expect(screen.getByText("800ms")).toBeInTheDocument();
    });
});
