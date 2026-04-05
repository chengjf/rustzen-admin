import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const getLatestForm = () => mocks.forms[mocks.forms.length - 1];

const mocks = vi.hoisted(() => ({
    createRole: vi.fn(),
    deleteRole: vi.fn(),
    error: vi.fn(),
    forms: [] as Array<{
        resetFields: ReturnType<typeof vi.fn>;
        setFieldsValue: ReturnType<typeof vi.fn>;
    }>,
    getOptionsWithCode: vi.fn(),
    success: vi.fn(),
    updateRole: vi.fn(),
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
        columns?: Array<{ key?: string; render?: (_: unknown, entity: any) => React.ReactNode }>;
    }) => (
        <div>
            <div>{headerTitle}</div>
            <div>{toolBarRender?.()}</div>
            <div>
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 2,
                        name: "管理员",
                        code: "admin",
                        menus: [{ value: 12 }],
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        name: "缺少ID角色",
                        code: "broken",
                        menus: [{ value: 12 }],
                    })}
            </div>
            <div>role-table</div>
        </div>
    ),
    ModalForm: ({
        children,
        title,
        trigger,
        onOpenChange,
        onFinish,
    }: {
        children?: React.ReactNode;
        title?: string;
        trigger?: React.ReactNode;
        onOpenChange?: (open: boolean) => void;
        onFinish?: (values: Record<string, unknown>) => Promise<boolean>;
    }) => (
        <div>
            {trigger}
            <div>{title}</div>
            <button aria-label={`open-${title}`} onClick={() => void onOpenChange?.(true)}>
                open-{title}
            </button>
            <button aria-label={`close-${title}`} onClick={() => void onOpenChange?.(false)}>
                close-{title}
            </button>
            <button
                aria-label={`submit-${title}`}
                onClick={() =>
                    void onFinish?.({
                        name: "角色",
                        code: "role",
                        menuIds: [12],
                    })
                }
            >
                submit-{title}
            </button>
            {children}
        </div>
    ),
    ProFormText: () => null,
    ProFormTextArea: () => null,
    ProFormSelect: () => null,
    ProFormDigit: () => null,
}));

vi.mock("antd", () => ({
    Button: ({ children }: { children: React.ReactNode }) => <button>{children}</button>,
    Form: Object.assign(({ children }: { children?: React.ReactNode }) => <div>{children}</div>, {
        useForm: () => {
            const form = {
                resetFields: vi.fn(),
                setFieldsValue: vi.fn(),
            };
            mocks.forms.push(form);
            return [form];
        },
        Item: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
    }),
    Space: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    Tree: ({
        treeData,
        onCheck,
        onSelect,
    }: {
        treeData?: Array<{ title?: React.ReactNode; key?: React.Key; children?: any[] }>;
        onCheck?: (info: { checked: React.Key[] }) => void;
        onSelect?: (keys: React.Key[]) => void;
    }) => {
        const flatten = (
            nodes: Array<{ title?: React.ReactNode; key?: React.Key; children?: any[] }>,
        ) => nodes.flatMap((node) => [node, ...(node.children ? flatten(node.children) : [])]);
        const items = flatten(treeData ?? []);

        return (
            <div>
                {items.map((node) => (
                    <div key={`tree-${String(node.key)}`}>
                        <span>{node.title}</span>
                        <button
                            aria-label={`select-${String(node.title)}`}
                            onClick={() => onSelect?.([node.key as React.Key])}
                        >
                            select-{node.title}
                        </button>
                        <button
                            aria-label={`check-${String(node.title)}`}
                            onClick={() => onCheck?.({ checked: [node.key as React.Key] })}
                        >
                            check-{node.title}
                        </button>
                    </div>
                ))}
            </div>
        );
    },
    Typography: { Text: ({ children }: { children: React.ReactNode }) => <span>{children}</span> },
    Checkbox: ({ checked, children }: { checked?: boolean; children?: React.ReactNode }) => (
        <div>
            {checked ? "checked:" : "unchecked:"}
            {children}
        </div>
    ),
    Card: ({ children, onClick }: { children: React.ReactNode; onClick?: () => void }) => (
        <button onClick={onClick}>{children}</button>
    ),
    Empty: Object.assign(
        ({ description }: { description?: React.ReactNode }) => <div>{description ?? "empty"}</div>,
        { PRESENTED_IMAGE_SIMPLE: "simple" },
    ),
    Row: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    Col: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    Spin: ({ tip }: { tip?: React.ReactNode }) => <div>{tip ?? "loading"}</div>,
}));

vi.mock("@/api", () => ({
    appMessage: { success: mocks.success, error: mocks.error },
}));

vi.mock("@/api/system/menu", () => ({
    menuAPI: { getOptionsWithCode: mocks.getOptionsWithCode },
}));

vi.mock("@/api/system/role", () => ({
    roleAPI: {
        create: mocks.createRole,
        delete: mocks.deleteRole,
        getTableData: vi.fn(),
        update: mocks.updateRole,
    },
}));

vi.mock("@/components/auth", () => ({
    AuthWrap: ({ code, children }: { code?: string; children?: React.ReactNode }) => {
        if (!code) return <>{children}</>;
        return useAuthStore.getState().checkPermissions(code) ? <>{children}</> : null;
    },
    AuthPopconfirm: ({
        children,
        onConfirm,
    }: {
        children?: React.ReactNode;
        onConfirm?: () => Promise<void>;
    }) => <button onClick={() => void onConfirm?.()}>{children}</button>,
}));

vi.mock("@/constant/options", () => ({
    ENABLE_DEFAULT: 1,
    ENABLE_OPTIONS: [],
    ENABLE_STATUS_ENUM: {},
}));

import { PermissionManager, RolePage } from "./role";

beforeEach(() => {
    mocks.forms.length = 0;
    mocks.getOptionsWithCode.mockResolvedValue([
        {
            value: 10,
            label: "系统管理",
            menuType: 1,
            children: [{ value: 12, label: "角色菜单", menuType: 2, parentId: 10 }],
        },
    ]);
    act(() => {
        useAuthStore.setState({
            token: "token",
            userInfo: {
                ...mockUserInfo,
                permissions: [
                    "system:role:list",
                    "system:role:create",
                    "system:role:update",
                    "system:role:delete",
                ],
            },
        });
    });
});

afterEach(() => {
    act(() => {
        useAuthStore.setState({ token: null, userInfo: null });
    });
    vi.clearAllMocks();
});

describe("RolePage", () => {
    it("renders the role page title and create button", () => {
        render(<RolePage />);

        expect(screen.getByText("角色权限管理")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "创建角色" })).toBeInTheDocument();
    });

    it("hides the create button without create permission", () => {
        act(() => {
            useAuthStore.setState({
                token: "token",
                userInfo: { ...mockUserInfo, permissions: ["system:role:list"] },
            });
        });

        render(<RolePage />);

        expect(screen.getByText("role-table")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "创建角色" })).not.toBeInTheDocument();
    });

    it("loads menu options and submits a create payload with parent menu ids", async () => {
        render(<RolePage />);

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "open-新建角色" }));
        });

        expect(mocks.getOptionsWithCode).toHaveBeenCalledWith({ btn_filter: false });

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "submit-新建角色" }));
        });

        expect(mocks.createRole).toHaveBeenCalledWith(
            expect.objectContaining({
                code: "role",
                menuIds: [12, 10],
                name: "角色",
            }),
        );
        expect(mocks.success).toHaveBeenCalledWith("创建成功");
    });

    it("hydrates edit form from loaded menus and submits update", async () => {
        render(<RolePage />);

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "open-编辑角色" })[0]);
        });

        expect(mocks.getOptionsWithCode).toHaveBeenCalledWith({ btn_filter: false });

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "submit-编辑角色" })[0]);
        });

        expect(mocks.updateRole).toHaveBeenCalledWith(
            2,
            expect.objectContaining({ menuIds: [12, 10] }),
        );
        expect(mocks.success).toHaveBeenCalledWith("更新成功");
    });

    it("resets the form when closing a modal and tolerates menu loading failures", async () => {
        vi.spyOn(console, "error").mockImplementation(() => {});
        mocks.getOptionsWithCode.mockRejectedValueOnce(new Error("load failed"));

        render(<RolePage />);

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "open-新建角色" }));
        });
        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "close-新建角色" }));
        });

        expect(getLatestForm()?.resetFields).toHaveBeenCalled();
        expect(console.error).toHaveBeenCalled();
    });

    it("deletes a role from the action column", async () => {
        mocks.deleteRole.mockResolvedValue(undefined);

        render(<RolePage />);

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "删除" })[0]);
        });

        expect(mocks.deleteRole).toHaveBeenCalledWith(2);
        expect(mocks.success).toHaveBeenCalledWith("删除成功");
    });

    it("shows an error when edit mode is missing an id", async () => {
        render(<RolePage />);

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "open-编辑角色" })[1]);
            fireEvent.click(screen.getAllByRole("button", { name: "submit-编辑角色" })[1]);
        });

        expect(mocks.error).toHaveBeenCalledWith("数据异常，请刷新后重试");
        expect(mocks.updateRole).not.toHaveBeenCalled();
    });
});

describe("PermissionManager", () => {
    const menuTree = [
        {
            value: 10,
            label: "系统管理",
            menuType: 1,
            children: [
                {
                    value: 12,
                    label: "角色菜单",
                    menuType: 2,
                    parentId: 10,
                    children: [
                        { value: 13, label: "查看按钮", menuType: 3, parentId: 12 },
                        { value: 14, label: "编辑按钮", menuType: 3, parentId: 12 },
                    ],
                },
                {
                    value: 15,
                    label: "无按钮菜单",
                    menuType: 2,
                    parentId: 10,
                },
            ],
        },
    ] as any;

    it("shows a loading state while menu data is fetching", () => {
        render(<PermissionManager loading menuTree={[]} />);

        expect(screen.getByText("加载菜单中...")).toBeInTheDocument();
    });

    it("adds parent menu ids when checking a child menu and keeps existing button ids", () => {
        const onChange = vi.fn();

        render(
            <PermissionManager
                loading={false}
                menuTree={menuTree}
                value={[13]}
                onChange={onChange}
            />,
        );

        fireEvent.click(screen.getByRole("button", { name: "check-角色菜单" }));

        expect(onChange).toHaveBeenCalledWith([12, 10, 13]);
    });

    it("shows button permissions for the selected menu and toggles them correctly", () => {
        const onChange = vi.fn();

        const { rerender } = render(
            <PermissionManager
                loading={false}
                menuTree={menuTree}
                value={[]}
                onChange={onChange}
            />,
        );

        expect(screen.getByText("请在左侧选择菜单")).toBeInTheDocument();

        fireEvent.click(screen.getByRole("button", { name: "select-角色菜单" }));

        expect(screen.getByText("功能配置：角色菜单")).toBeInTheDocument();
        expect(screen.getByText("unchecked:查看按钮")).toBeInTheDocument();
        expect(screen.getByText("unchecked:编辑按钮")).toBeInTheDocument();

        fireEvent.click(screen.getByRole("button", { name: "unchecked:查看按钮" }));

        expect(onChange).toHaveBeenCalledWith(expect.arrayContaining([10, 12, 13]));

        rerender(
            <PermissionManager
                loading={false}
                menuTree={menuTree}
                value={[10, 12, 13]}
                onChange={onChange}
            />,
        );

        fireEvent.click(screen.getByRole("button", { name: "select-角色菜单" }));
        fireEvent.click(screen.getByRole("button", { name: "checked:查看按钮" }));

        expect(onChange).toHaveBeenCalledWith([10, 12]);
    });

    it("shows empty states for menus without buttons", () => {
        render(<PermissionManager loading={false} menuTree={menuTree} value={[]} />);

        fireEvent.click(screen.getByRole("button", { name: "select-无按钮菜单" }));

        expect(screen.getByText("该菜单下无功能按钮")).toBeInTheDocument();
    });
});
