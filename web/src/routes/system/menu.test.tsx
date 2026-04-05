import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const mocks = vi.hoisted(() => ({
    createMenu: vi.fn(),
    deleteMenu: vi.fn(),
    error: vi.fn(),
    forms: [] as Array<{
        resetFields: ReturnType<typeof vi.fn>;
        setFieldsValue: ReturnType<typeof vi.fn>;
    }>,
    treeRequests: [] as Array<
        () => Promise<Array<{ children?: Array<{ label: string; value: number }> }>>
    >,
    success: vi.fn(),
    updateMenu: vi.fn(),
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
                    ?.find((column) => column.dataIndex === "menuType")
                    ?.render?.(null, {
                        id: 4,
                        menuType: 1,
                    })}
                {columns
                    ?.find((column) => column.dataIndex === "menuType")
                    ?.render?.(null, {
                        id: 7,
                        menuType: 3,
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 5,
                        isSystem: false,
                        name: "普通菜单",
                        parentId: 0,
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        id: 6,
                        isSystem: true,
                        name: "系统菜单",
                        parentId: 0,
                    })}
                {columns
                    ?.find((column) => column.key === "action")
                    ?.render?.(null, {
                        isSystem: false,
                        name: "缺失ID菜单",
                        parentId: 0,
                    })}
            </div>
            <div>menu-table</div>
        </div>
    ),
    ProFormTreeSelect: ({
        request,
    }: {
        request?: () => Promise<Array<{ children?: Array<{ label: string; value: number }> }>>;
    }) => {
        if (request) mocks.treeRequests.push(request);
        return null;
    },
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
            <button aria-label={`open-${title}`} onClick={() => onOpenChange?.(true)}>
                open-{title}
            </button>
            <button aria-label={`close-${title}`} onClick={() => onOpenChange?.(false)}>
                close-{title}
            </button>
            <button
                aria-label={`submit-${title}`}
                onClick={() =>
                    void onFinish?.({
                        code: "system:menu:list",
                        menuType: 2,
                        name: "菜单项",
                        parentId: 0,
                        status: 1,
                    })
                }
            >
                submit-{title}
            </button>
            {children}
        </div>
    ),
    ProFormDigit: () => null,
    ProFormSelect: () => null,
    ProFormText: () => null,
}));

vi.mock("antd", () => ({
    Button: ({ children }: { children: React.ReactNode }) => <button>{children}</button>,
    Space: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    Tag: ({ children }: { children: React.ReactNode }) => <span>{children}</span>,
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
}));

vi.mock("@/api", () => ({
    appMessage: { success: mocks.success, error: mocks.error },
}));

vi.mock("@/api/system/menu", () => ({
    menuAPI: {
        create: mocks.createMenu,
        update: mocks.updateMenu,
        delete: mocks.deleteMenu,
        getOptionsWithCode: vi.fn().mockResolvedValue([
            {
                label: "系统管理",
                value: 1,
                menuType: 1,
                children: [
                    { label: "菜单管理", value: 2, menuType: 2 },
                    { label: "删除按钮", value: 3, menuType: 3 },
                ],
            },
        ]),
        getTableData: vi.fn(),
    },
}));

vi.mock("@/components/auth", () => ({
    AuthWrap: ({ code, children }: { code?: string; children?: React.ReactNode }) => {
        if (!code) return <>{children}</>;
        return useAuthStore.getState().checkPermissions(code) ? <>{children}</> : null;
    },
    AuthPopconfirm: ({
        children,
        hidden,
        onConfirm,
    }: {
        children?: React.ReactNode;
        hidden?: boolean;
        onConfirm?: () => Promise<void>;
    }) => (hidden ? null : <button onClick={() => void onConfirm?.()}>{children}</button>),
}));

vi.mock("@/constant/options", () => ({
    ENABLE_DEFAULT: 1,
    ENABLE_OPTIONS: [],
    ENABLE_STATUS_ENUM: {},
    MENU_TYPE_OPTIONS: [],
}));

import { MenuPage } from "./menu";

beforeEach(() => {
    mocks.forms.length = 0;
    mocks.treeRequests.length = 0;
    act(() => {
        useAuthStore.setState({
            token: "token",
            userInfo: {
                ...mockUserInfo,
                permissions: [
                    "system:menu:list",
                    "system:menu:create",
                    "system:menu:update",
                    "system:menu:delete",
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

describe("MenuPage", () => {
    it("renders the menu page title and create button", () => {
        render(<MenuPage />);

        expect(screen.getByText("菜单权限架构")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "创建菜单" })).toBeInTheDocument();
    });

    it("hides the create button without create permission", () => {
        act(() => {
            useAuthStore.setState({
                token: "token",
                userInfo: { ...mockUserInfo, permissions: ["system:menu:list"] },
            });
        });

        render(<MenuPage />);

        expect(screen.getByText("menu-table")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "创建菜单" })).not.toBeInTheDocument();
    });

    it("filters button nodes from the parent menu tree options", async () => {
        render(<MenuPage />);

        const result = await mocks.treeRequests[0]?.();

        expect(result).toEqual([
            {
                label: "顶级根目录",
                value: 0,
                children: [
                    {
                        label: "系统管理",
                        value: 1,
                        menuType: 1,
                        children: [{ label: "菜单管理", value: 2, menuType: 2, children: null }],
                    },
                ],
            },
        ]);
    });

    it("submits the create and edit menu flows", async () => {
        render(<MenuPage />);

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "submit-新增菜单节点" }));
        });
        expect(mocks.createMenu).toHaveBeenCalledWith(
            expect.objectContaining({
                code: "system:menu:list",
                menuType: 2,
                name: "菜单项",
            }),
        );
        expect(mocks.success).toHaveBeenCalledWith("创建成功");

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "open-编辑菜单属性" })[0]);
            fireEvent.click(screen.getAllByRole("button", { name: "submit-编辑菜单属性" })[0]);
        });

        expect(mocks.forms[1]?.setFieldsValue).toHaveBeenCalledWith(
            expect.objectContaining({ id: 5, name: "普通菜单" }),
        );
        expect(mocks.updateMenu).toHaveBeenCalledWith(
            5,
            expect.objectContaining({ name: "菜单项" }),
        );
        expect(mocks.success).toHaveBeenCalledWith("更新保存成功");
    });

    it("renders menu type tags, resets the form on close, and guards edit without id", async () => {
        render(<MenuPage />);

        expect(screen.getByText("目录")).toBeInTheDocument();
        expect(screen.getByText("按钮")).toBeInTheDocument();

        await act(async () => {
            fireEvent.click(screen.getByRole("button", { name: "open-新增菜单节点" }));
            fireEvent.click(screen.getByRole("button", { name: "close-新增菜单节点" }));
        });

        expect(mocks.forms[0]?.setFieldsValue).toHaveBeenCalledWith(
            expect.objectContaining({
                menuType: 1,
                parentId: 0,
                sortOrder: 0,
                status: 1,
            }),
        );
        expect(mocks.forms[0]?.resetFields).toHaveBeenCalled();

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "open-编辑菜单属性" })[2]);
            fireEvent.click(screen.getAllByRole("button", { name: "submit-编辑菜单属性" })[2]);
        });

        expect(mocks.error).toHaveBeenCalledWith("数据异常：缺失 ID");
        expect(mocks.updateMenu).not.toHaveBeenCalled();
    });

    it("hides delete for system menus and deletes normal menus", async () => {
        mocks.deleteMenu.mockResolvedValue(undefined);

        render(<MenuPage />);

        expect(screen.getAllByRole("button", { name: "删除" }).length).toBeGreaterThanOrEqual(1);

        await act(async () => {
            fireEvent.click(screen.getAllByRole("button", { name: "删除" })[0]);
        });

        expect(mocks.deleteMenu).toHaveBeenCalledWith(5);
        expect(mocks.success).toHaveBeenCalledWith("删除成功");
    });
});
