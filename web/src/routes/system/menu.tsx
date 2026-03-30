import {
    type ActionType,
    type ProColumns,
    ProFormTreeSelect,
    ProTable,
    ModalForm,
    ProFormDigit,
    ProFormSelect,
    ProFormText,
} from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Space, Tag, Form } from "antd";
import React, { useRef, useCallback, useMemo } from "react";

import { appMessage } from "@/api";
import { menuAPI } from "@/api/system/menu";
import type { MenuItemResp } from "@/api/types/MenuItemResp";
import type { MenuQuery } from "@/api/types/MenuQuery";
import type { MenuTreeOption } from "@/api/types/MenuTreeOption";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";
import { ENABLE_OPTIONS, MENU_TYPE_OPTIONS } from "@/constant/options";

// =============================================================================
// 1. 路由定义 (Router Definition) - 放置于顶部以兼容路由 Codegen
// =============================================================================

export const Route = createFileRoute("/system/menu")({
    component: MenuPage,
});

// =============================================================================
// 2. 常量与工具函数 (Constants & Helpers)
// =============================================================================

const MENU_TYPE_CONFIG: Record<number, { text: string; color: string }> = {
    1: { text: "目录", color: "cyan" },
    2: { text: "菜单", color: "purple" },
    3: { text: "按钮", color: "orange" },
};

/**
 * 递归过滤树节点中的按钮类型
 */
const filterButtonNodes = (nodes: MenuTreeOption[]): MenuTreeOption[] => {
    return nodes
        .filter((node) => node.menuType !== 3)
        .map(
            (node): MenuTreeOption => ({
                ...node,
                children:
                    node.children && node.children.length > 0
                        ? filterButtonNodes(node.children)
                        : null,
            }),
        );
};

// =============================================================================
// 3. 子组件 (Sub-Components) - 声明于主组件之前
// =============================================================================

interface MenuModalFormProps {
    initialValues?: Partial<MenuItemResp>;
    mode?: "create" | "edit";
    children: React.ReactNode;
    onSuccess?: () => void;
}

const MenuModalForm = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}: MenuModalFormProps) => {
    const [form] = Form.useForm();

    const handleOpenChange = (open: boolean) => {
        if (open && initialValues) {
            form.setFieldsValue(initialValues);
        } else if (!open) {
            form.resetFields();
        }
    };

    return (
        <ModalForm
            form={form}
            width={580}
            title={mode === "create" ? "新增菜单节点" : "编辑菜单属性"}
            trigger={children}
            layout="horizontal"
            labelCol={{ span: 5 }}
            wrapperCol={{ span: 18 }}
            modalProps={{
                destroyOnClose: true,
                maskClosable: false,
            }}
            onOpenChange={handleOpenChange}
            onFinish={async (values) => {
                try {
                    if (mode === "create") {
                        await menuAPI.create(values);
                        appMessage.success("创建成功");
                    } else if (mode === "edit") {
                        if (!initialValues?.id) {
                            appMessage.error("数据异常：缺失 ID");
                            return false;
                        }
                        await menuAPI.update(initialValues.id, values);
                        appMessage.success("更新保存成功");
                    }
                    onSuccess?.();
                    return true;
                } catch (error) {
                    console.error("[MenuModalForm Submit Error]:", error);
                    return false;
                }
            }}
        >
            <ProFormTreeSelect
                name="parentId"
                label="上级菜单"
                request={async () => {
                    const res = await menuAPI.getOptionsWithCode({ btn_filter: true });
                    return [
                        {
                            label: "顶级根目录",
                            value: 0,
                            children: filterButtonNodes(res),
                        },
                    ];
                }}
                fieldProps={{
                    showSearch: true,
                    treeNodeFilterProp: "label",
                    treeDefaultExpandAll: true,
                }}
                rules={[{ required: true, message: "请指定上级菜单" }]}
            />

            <ProFormSelect
                label="菜单类型"
                name="menuType"
                options={MENU_TYPE_OPTIONS}
                rules={[{ required: true }]}
            />

            <ProFormText name="name" label="名称" rules={[{ required: true, max: 32 }]} />

            <ProFormText name="code" label="权限标识" rules={[{ required: true }]} />

            <ProFormSelect
                name="status"
                label="状态"
                options={ENABLE_OPTIONS}
                rules={[{ required: true }]}
            />

            <ProFormDigit
                name="sortOrder"
                label="显示排序"
                min={0}
                initialValue={0}
                fieldProps={{ precision: 0 }}
            />
        </ModalForm>
    );
};

// =============================================================================
// 4. 页面主组件 (Main Component)
// =============================================================================

function MenuPage() {
    const actionRef = useRef<ActionType>(null);

    const handleReload = useCallback(() => {
        void actionRef.current?.reload();
    }, []);

    const columns: ProColumns<MenuItemResp>[] = useMemo(
        () => [
            { title: "名称", dataIndex: "name", width: 200, ellipsis: true },
            { title: "编码", align: "center", width: 150, dataIndex: "code", ellipsis: true },
            {
                title: "类型",
                align: "center",
                dataIndex: "menuType",
                width: 100,
                valueEnum: MENU_TYPE_CONFIG,
                render: (_, record) => {
                    const config = MENU_TYPE_CONFIG[record.menuType];
                    return <Tag color={config?.color}>{config?.text}</Tag>;
                },
            },
            {
                title: "状态",
                align: "center",
                dataIndex: "status",
                width: 100,
                valueEnum: {
                    1: { text: "启用", status: "Success" },
                    2: { text: "禁用", status: "Default" },
                },
            },
            { title: "排序", align: "center", dataIndex: "sortOrder", width: 80, search: false },
            {
                title: "更新时间",
                align: "center",
                dataIndex: "updatedAt",
                valueType: "dateTime",
                width: 180,
                search: false,
            },
            {
                title: "操作",
                align: "center",
                key: "action",
                width: 160,
                fixed: "right",
                render: (_, entity) => (
                    <Space size="small">
                        <AuthWrap code="system:menu:update">
                            <MenuModalForm
                                mode="edit"
                                initialValues={entity}
                                onSuccess={handleReload}
                            >
                                <a>编辑</a>
                            </MenuModalForm>
                        </AuthWrap>
                        <AuthPopconfirm
                            code="system:menu:delete"
                            title="确定要删除此菜单吗？"
                            description="此操作将同步删除下级菜单且不可撤销。"
                            hidden={entity.isSystem}
                            onConfirm={async () => {
                                try {
                                    await menuAPI.delete(entity.id);
                                    appMessage.success("删除成功");
                                    handleReload();
                                } catch (e) {
                                    console.error("[Delete Menu Error]:", e);
                                }
                            }}
                        >
                            <span className="cursor-pointer text-red-500">删除</span>
                        </AuthPopconfirm>
                    </Space>
                ),
            },
        ],
        [handleReload],
    );

    const toolBarRender = useCallback(
        () => [
            <AuthWrap code="system:menu:create" key="add">
                <MenuModalForm
                    mode="create"
                    initialValues={{ sortOrder: 0, status: 1, menuType: 1, parentId: 0 }}
                    onSuccess={handleReload}
                >
                    <Button type="primary">创建菜单</Button>
                </MenuModalForm>
            </AuthWrap>,
        ],
        [handleReload],
    );

    return (
        <AuthWrap code="system:menu:list">
            <ProTable<MenuItemResp>
                rowKey="id"
                search={{ labelWidth: "auto" }}
                scroll={{ y: "calc(100vh - 280px)" }}
                headerTitle="菜单权限架构"
                columns={columns}
                request={async (params) => {
                    const res = await menuAPI.getTableData(params as Partial<MenuQuery>);
                    return { data: res, success: true };
                }}
                actionRef={actionRef}
                pagination={false}
                toolBarRender={toolBarRender}
            />
        </AuthWrap>
    );
}
