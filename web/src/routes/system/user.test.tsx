import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const getLatestForm = () => mocks.forms[mocks.forms.length - 1];

const mocks = vi.hoisted(() => ({
    createUser: vi.fn(),
    deleteUser: vi.fn(),
    forms: [] as Array<{
        resetFields: ReturnType<typeof vi.fn>;
        setFieldsValue: ReturnType<typeof vi.fn>;
    }>,
    resetPassword: vi.fn(),
    success: vi.fn(),
    unlockUser: vi.fn(),
    updateUser: vi.fn(),
    updateStatus: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
    createFileRoute: () => () => ({}),
}));

vi.mock("@ant-design/pro-components", () => ({
    ProTable: ({
        headerTitle,
        toolBarRender,
        columns,
    }: {
        headerTitle?: React.ReactNode;
        toolBarRender?: () => React.ReactNode[];
        columns?: Array<{
            dataIndex?: string;
            key?: string;
            render?: (_: unknown, entity: any) => React.ReactNode;
        }>;
    }) => (
        <div>
            <div>{headerTitle}</div>
            <div>{toolBarRender?.()}</div>
            <div>
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 2,
                        username: "normal-user",
                        email: "normal@test.dev",
                        status: "Normal",
                        lockExpiresAt: null,
                        roles: [{ label: "管理员", value: 9 }],
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 3,
                        username: "disabled-user",
                        status: "Disabled",
                        lockExpiresAt: null,
                        roles: [],
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 4,
                        username: "locked-user",
                        status: "Locked",
                        lockExpiresAt: "2099-01-01T00:00:00.000Z",
                        roles: [],
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 7,
                        username: "self-user",
                        status: "Normal",
                        lockExpiresAt: null,
                        roles: [],
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 1,
                        username: "root-user",
                        status: "Normal",
                        lockExpiresAt: null,
                        roles: [],
                    })}
                <div>
                    {columns
                        ?.find((column) => column.dataIndex === "status")
                        ?.render?.(null, {
                            id: 4,
                            username: "locked-user",
                            status: "Locked",
                            lockExpiresAt: "2099-01-01T00:00:00.000Z",
                            roles: [],
                        })}
                </div>
                <div>
                    {columns
                        ?.find((column) => column.dataIndex === "status")
                        ?.render?.(null, {
                            id: 5,
                            username: "pending-user",
                            status: "Pending",
                            lockExpiresAt: null,
                            roles: [],
                        })}
                </div>
            </div>
            <div>user-table</div>
        </div>
    ),
    ModalForm: ({
        children,
        title,
        open,
        onOpenChange,
        onFinish,
    }: {
        children?: React.ReactNode;
        title?: string;
        open?: boolean;
        onOpenChange?: (open: boolean) => void;
        onFinish?: (values: Record<string, unknown>) => Promise<boolean>;
    }) => (
        <div>
            {open ? <div>{title}</div> : null}
            {open ? (
                <>
                    <button
                        aria-label={`submit-${title}`}
                        onClick={() =>
                            void onFinish?.({
                                username: "alice",
                                email: "alice@test.dev",
                                password: "secret123",
                                roleIds: [1],
                            })
                        }
                    >
                        submit-{title}
                    </button>
                    <button aria-label={`close-${title}`} onClick={() => onOpenChange?.(false)}>
                        close-{title}
                    </button>
                </>
            ) : null}
            {children}
        </div>
    ),
    ProFormSelect: () => null,
    ProFormText: Object.assign(() => null, { Password: () => null }),
}));

vi.mock("antd", () => ({
    Button: ({ children, onClick }: { children: React.ReactNode; onClick?: () => void }) => (
        <button onClick={onClick}>{children}</button>
    ),
    Form: {
        useForm: () => {
            const form = {
                resetFields: vi.fn(),
                setFieldsValue: vi.fn(),
            };
            mocks.forms.push(form);
            return [form];
        },
    },
    Modal: ({
        title,
        children,
        open,
        onOk,
        onCancel,
    }: {
        title?: React.ReactNode;
        children?: React.ReactNode;
        open?: boolean;
        onOk?: () => void;
        onCancel?: () => void;
    }) =>
        open ? (
            <div>
                <div>{title}</div>
                <button aria-label="confirm-password-modal" onClick={onOk}>
                    ok
                </button>
                <button aria-label="cancel-password-modal" onClick={onCancel}>
                    cancel
                </button>
                {children}
            </div>
        ) : null,
    Space: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    Typography: { Text: ({ children }: { children: React.ReactNode }) => <span>{children}</span> },
    Avatar: () => <span>avatar</span>,
    Tooltip: ({ children, title }: { children: React.ReactNode; title?: React.ReactNode }) => (
        <div>
            <div>{title}</div>
            {children}
        </div>
    ),
}));

vi.mock("@/api", () => ({
    appMessage: { success: mocks.success, error: vi.fn() },
}));

vi.mock("@/api/system/role", () => ({
    roleAPI: { getOptions: vi.fn() },
}));

vi.mock("@/api/system/user", () => ({
    userAPI: {
        create: mocks.createUser,
        update: mocks.updateUser,
        unlock: mocks.unlockUser,
        resetPassword: mocks.resetPassword,
        delete: mocks.deleteUser,
        updateStatus: mocks.updateStatus,
        getTableData: vi.fn(),
    },
}));

vi.mock("@/components/auth", () => ({
    AuthWrap: ({ code, children }: { code?: string; children?: React.ReactNode }) => {
        if (!code) return <>{children}</>;
        return useAuthStore.getState().checkPermissions(code) ? <>{children}</> : null;
    },
    AuthConfirm: ({
        children,
        hidden,
        onConfirm,
    }: {
        children?: React.ReactNode;
        hidden?: boolean;
        onConfirm?: () => Promise<void>;
    }) => (hidden ? null : <button onClick={() => void onConfirm?.()}>{children}</button>),
}));

vi.mock("@/components/button", () => ({
    MoreButton: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock("@/integrations/react-query", () => ({
    useApiQuery: () => ({ data: [] }),
}));

import { UserPage } from "./user";

beforeEach(() => {
    mocks.forms.length = 0;
    act(() => {
        useAuthStore.setState({
            token: "token",
            userInfo: {
                ...mockUserInfo,
                id: 7,
                permissions: [
                    "system:user:list",
                    "system:user:create",
                    "system:user:update",
                    "system:user:status",
                    "system:user:unlock",
                    "system:user:password",
                    "system:user:delete",
                ],
            },
        });
    });
    mocks.resetPassword.mockResolvedValue({ password: "Temp@123" });
});

afterEach(() => {
    act(() => {
        useAuthStore.setState({ token: null, userInfo: null });
    });
    vi.clearAllMocks();
});

describe("UserPage", () => {
    it("renders the table and create button when permissions allow", () => {
        render(<UserPage />);

        expect(screen.getByText("user-table")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "创建用户" })).toBeInTheDocument();
    });

    it("hides the create button without create permission", () => {
        act(() => {
            useAuthStore.setState({
                token: "token",
                userInfo: { ...mockUserInfo, permissions: ["system:user:list"] },
            });
        });

        render(<UserPage />);

        expect(screen.getByText("user-table")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "创建用户" })).not.toBeInTheDocument();
    });

    it("opens the create and edit user modal flows", async () => {
        render(<UserPage />);

        fireEvent.click(screen.getByRole("button", { name: "创建用户" }));
        expect(screen.getAllByText("创建用户")).toHaveLength(2);
        await waitFor(() => {
            expect(getLatestForm()?.resetFields).toHaveBeenCalled();
        });
        expect(getLatestForm()?.setFieldsValue).toHaveBeenCalledWith({ status: "Normal" });

        fireEvent.click(screen.getAllByText("编辑")[0]);

        await waitFor(() => {
            expect(screen.getByText("编辑用户")).toBeInTheDocument();
        });
    });

    it("hydrates edit values and submits create/update modal flows", async () => {
        render(<UserPage />);

        fireEvent.click(screen.getByRole("button", { name: "创建用户" }));
        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "submit-创建用户" }));
        });

        expect(mocks.createUser).toHaveBeenCalledWith(
            expect.objectContaining({
                email: "alice@test.dev",
                password: "secret123",
                roleIds: [1],
                username: "alice",
            }),
        );
        expect(mocks.success).toHaveBeenCalledWith("创建用户成功");

        fireEvent.click(screen.getAllByText("编辑")[0]);

        await waitFor(() => {
            expect(getLatestForm()?.setFieldsValue).toHaveBeenCalledWith(
                expect.objectContaining({
                    email: "normal@test.dev",
                    roleIds: [9],
                    username: "normal-user",
                }),
            );
        });

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "submit-编辑用户" }));
        });

        expect(mocks.updateUser).toHaveBeenCalledWith(
            2,
            expect.objectContaining({
                email: "alice@test.dev",
                roleIds: [1],
            }),
        );
        expect(mocks.success).toHaveBeenCalledWith("更新用户成功");
    });

    it("updates user status, unlocks locked users, and deletes users", async () => {
        render(<UserPage />);

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "禁用" }));
            fireEvent.click(screen.getByRole("button", { name: "启用" }));
            fireEvent.click(screen.getByRole("button", { name: "解除锁定" }));
            fireEvent.click(screen.getAllByRole("button", { name: "删除用户" })[0]);
        });

        expect(mocks.updateStatus).toHaveBeenNthCalledWith(1, 2, { status: "Disabled" });
        expect(mocks.updateStatus).toHaveBeenNthCalledWith(2, 3, { status: "Normal" });
        expect(mocks.unlockUser).toHaveBeenCalledWith(4);
        expect(mocks.deleteUser).toHaveBeenCalledWith(2);
        expect(mocks.success).toHaveBeenCalledWith("已禁用用户");
        expect(mocks.success).toHaveBeenCalledWith("已启用用户");
        expect(mocks.success).toHaveBeenCalledWith("已解除锁定");
        expect(mocks.success).toHaveBeenCalledWith("删除用户成功");
    });

    it("shows the reset password modal after a successful reset", async () => {
        render(<UserPage />);

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "重置密码" })[0]);
        });

        expect(mocks.resetPassword).toHaveBeenCalledWith(2);
        expect(mocks.success).not.toHaveBeenCalled();
        expect(screen.getByText("密码重置成功")).toBeInTheDocument();
        expect(screen.getByText("Temp@123")).toBeInTheDocument();
    });

    it("closes the password modal from both ok and cancel actions", async () => {
        const { rerender } = render(<UserPage />);

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "重置密码" })[0]);
        });
        fireEvent.click(screen.getByRole("button", { name: "confirm-password-modal" }));

        expect(screen.queryByText("密码重置成功")).not.toBeInTheDocument();

        rerender(<UserPage />);

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "重置密码" })[0]);
        });
        fireEvent.click(screen.getByRole("button", { name: "cancel-password-modal" }));

        expect(screen.queryByText("密码重置成功")).not.toBeInTheDocument();
    });

    it("renders locked-user tooltip text and hides actions for self and root users", () => {
        vi.spyOn(Date, "now").mockReturnValue(new Date("2098-12-31T23:30:00.000Z").getTime());

        render(<UserPage />);

        expect(screen.getByText("自动锁定，约 30 分钟后解锁")).toBeInTheDocument();
        expect(screen.getByText("锁定")).toBeInTheDocument();
        expect(screen.getByText("待审核")).toBeInTheDocument();
        expect(screen.getAllByText("编辑")).toHaveLength(3);
    });

    it("keeps the modal open and suppresses success toast when create or update fails", async () => {
        vi.spyOn(console, "error").mockImplementation(() => {});
        mocks.createUser.mockRejectedValueOnce(new Error("create failed"));
        mocks.updateUser.mockRejectedValueOnce(new Error("update failed"));

        render(<UserPage />);

        fireEvent.click(screen.getByRole("button", { name: "创建用户" }));
        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "submit-创建用户" }));
        });

        expect(mocks.success).not.toHaveBeenCalledWith("创建用户成功");
        expect(console.error).toHaveBeenCalled();

        fireEvent.click(screen.getByRole("button", { name: "close-创建用户" }));
        fireEvent.click(screen.getAllByText("编辑")[0]);
        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "submit-编辑用户" }));
        });

        expect(mocks.success).not.toHaveBeenCalledWith("更新用户成功");
    });
});
