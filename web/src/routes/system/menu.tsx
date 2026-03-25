import type { ActionType, ProColumns } from "@ant-design/pro-components";
import { ProFormTreeSelect, ProTable } from "@ant-design/pro-components";
import { ModalForm, ProFormDigit, ProFormSelect, ProFormText } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Space, Tag } from "antd";
import { Form } from "antd";
import React, { useRef } from "react";

import { menuAPI } from "@/api/system/menu";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";
import { ENABLE_OPTIONS, MENU_TYPE_OPTIONS } from "@/constant/options";

export const Route = createFileRoute("/system/menu")({
    component: MenuPage,
});

function MenuPage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Menu.Item>
            rowKey="id"
            search={{
                labelWidth: "auto",
            }}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="菜单管理"
            columns={columns}
            request={async (params) => {
                const res = await menuAPI.getTableData(params);
                return {
                    data: res,
                    success: true,
                };
            }}
            actionRef={actionRef}
            pagination={false}
            toolBarRender={() => [
                <AuthWrap code="system:menu:create">
                    <MenuModalForm
                        mode={"create"}
                        initialValues={{ sortOrder: 0 }}
                        onSuccess={() => {
                            actionRef.current?.reload();
                        }}
                    >
                        <Button type="primary">创建菜单</Button>
                    </MenuModalForm>
                </AuthWrap>,
            ]}
        />
    );
}

const menuTypeEnum: Record<number, { text: string; color: string }> = {
    1: { text: "目录", color: "cyan" },
    2: { text: "菜单", color: "purple" },
    3: { text: "按钮", color: "warning" },
};

const columns: ProColumns<Menu.Item>[] = [
    {
        title: "",
        dataIndex: "",
        width: 60,
        hideInSearch: true,
    },
    {
        title: "名称",
        align: "center",
        width: 120,
        dataIndex: "name",
        ellipsis: true,
    },
    {
        title: "编码",
        align: "center",
        width: 120,
        dataIndex: "code",
        ellipsis: true,
    },
    {
        title: "菜单类型",
        align: "center",
        dataIndex: "menuType",
        width: 120,
        ellipsis: true,
        valueEnum: {
            1: { text: "目录", color: "cyan" },
            2: { text: "菜单", color: "purple" },
            3: { text: "按钮", color: "warning" },
        },
        render: (_, record) => {
            const item = menuTypeEnum[record.menuType];
            return <Tag color={item.color}>{item.text}</Tag>;
        },
    },
    {
        title: "状态",
        align: "center",
        dataIndex: "status",
        width: 120,
        ellipsis: true,
        valueEnum: {
            1: { text: "启用", status: "Success" },
            2: { text: "禁用", status: "Default" },
        },
    },
    {
        title: "排序",
        align: "center",
        dataIndex: "sortOrder",
        width: 120,
        ellipsis: true,
        hideInSearch: true,
    },
    {
        title: "更新时间",
        align: "center",
        dataIndex: "updatedAt",
        valueType: "dateTime",
        width: 200,
        hideInSearch: true,
    },
    {
        title: "操作",
        align: "center",
        key: "action",
        width: 200,
        fixed: "right",
        render: (_dom: React.ReactNode, entity: Menu.Item, _index, action?: ActionType) => (
            <Space size="middle">
                <AuthWrap code="system:menu:update">
                    <MenuModalForm
                        mode={"edit"}
                        initialValues={entity}
                        onSuccess={() => {
                            action?.reload();
                        }}
                    >
                        <a>编辑</a>
                    </MenuModalForm>
                </AuthWrap>
                <AuthPopconfirm
                    code="system:menu:delete"
                    title="确定要删除此菜单吗？"
                    description="此操作不可撤销。"
                    hidden={entity.isSystem}
                    onConfirm={async () => {
                        await menuAPI.delete(entity.id);
                        action?.reload();
                    }}
                >
                    <span className="cursor-pointer text-red-500">删除</span>
                </AuthPopconfirm>
            </Space>
        ),
    },
];

interface MenuModalFormProps {
    initialValues?: Partial<Menu.Item>;
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

    return (
        <ModalForm<Menu.CreateAndUpdateRequest>
            form={form}
            width={600}
            layout="horizontal"
            title={mode === "create" ? "创建菜单" : "编辑菜单"}
            trigger={children}
            labelCol={{ span: 6 }}
            wrapperCol={{ span: 18 }}
            modalProps={{
                destroyOnHidden: true,
                maskClosable: false,
                okText: mode === "create" ? "创建" : "保存",
                cancelText: "取消",
            }}
            onOpenChange={(open) => {
                if (open) {
                    form.setFieldsValue(initialValues);
                } else {
                    form.resetFields();
                }
            }}
            onFinish={async (values) => {
                if (mode === "create") {
                    await menuAPI.create(values as Menu.CreateAndUpdateRequest);
                } else if (mode === "edit" && initialValues?.id) {
                    await menuAPI.update(initialValues.id, values as Menu.CreateAndUpdateRequest);
                }
                onSuccess?.();
                return true;
            }}
        >
            <ProFormTreeSelect
                name="parentId"
                label="上级菜单"
                placeholder="请选择上级菜单"
                request={async () => {
                    const res = await menuAPI.getOptionsWithCode({ btn_filter: true });
                    // 添加默认根节点id为0
                    console.log(res);
                    let root = { label: "根菜单", value: 0, children: [] as Api.MenuTreeOption[] };
                    // 过滤点menuType为3的，递归children

                    // 将res中的parentId为0的项添加到root的children中
                    let rootChild = res.filter((item) => item.parentId === 0);
                    root.children = rootChild;
                    return [root];
                }}
                fieldProps={{
                    showSearch: true,
                    // 关键配置：指定搜索时过滤哪一个字段
                    treeNodeFilterProp: "label",
                    // 建议同时开启此项，支持搜索子节点时展示层级
                    treeDefaultExpandAll: true,
                }}
                rules={[{ required: true, message: "请选择上级菜单" }]}
            />
            <ProFormText
                name="name"
                label="菜单名称"
                placeholder="请输入菜单名称"
                rules={[{ required: true, message: "请输入菜单名称" }]}
            />
            <ProFormText
                name="code"
                label="权限编码"
                placeholder="请输入权限编码（如 system:menu:list）"
                rules={[{ required: true, message: "请输入权限编码" }]}
            />
            <ProFormSelect
                label="类型"
                name="menuType"
                options={MENU_TYPE_OPTIONS}
                rules={[{ required: true, message: "请选择菜单类型" }]}
            />
            <ProFormSelect
                name="status"
                label="状态"
                placeholder="请选择状态"
                options={ENABLE_OPTIONS}
                rules={[{ required: true, message: "请选择状态" }]}
            />
            <ProFormDigit
                name="sortOrder"
                label="排序"
                placeholder="请输入排序"
                min={0}
                fieldProps={{ precision: 0 }}
            />
        </ModalForm>
    );
};
