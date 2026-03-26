import type { ActionType, ProColumns } from "@ant-design/pro-components";
import { ProTable } from "@ant-design/pro-components";
import { ModalForm, ProFormSelect, ProFormText } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Space, Form } from "antd";
import React, { useRef } from "react";

import { roleAPI } from "@/api/system/role";
import { userAPI } from "@/api/system/user";
import type { UserItemResp, CreateUserDto, UpdateUserPayload } from "@/api/types";
import { AuthConfirm, AuthWrap } from "@/components/auth";
import { MoreButton } from "@/components/button";
import { useApiQuery } from "@/integrations/react-query";
import { useAuthStore } from "@/stores/useAuthStore";

export const Route = createFileRoute("/system/user")({
    component: UserPage,
});

function UserPage() {
    const actionRef = useRef<ActionType>(null);
    return (
        <ProTable<UserItemResp>
            rowKey="id"
            scroll={{ y: "calc(100vh - 383px)" }}
            headerTitle="用户列表"
            columns={columns}
            request={userAPI.getTableData}
            actionRef={actionRef}
            search={{ span: 6 }}
            toolBarRender={() => [
                <AuthWrap code="system:user:create">
                    <UserModalForm
                        mode={"create"}
                        onSuccess={() => {
                            void actionRef.current?.reload();
                        }}
                        initialValues={{
                            status: 1,
                        }}
                    >
                        <Button type="primary">创建用户</Button>
                    </UserModalForm>
                </AuthWrap>,
            ]}
        />
    );
}

const columns: ProColumns<UserItemResp>[] = [
    {
        title: "ID",
        align: "center",
        dataIndex: "id",
        width: 48,
        search: false,
    },
    {
        title: "头像",
        align: "center",
        dataIndex: "avatarUrl",
        width: 60,
        search: false,
        render: (_, record) => {
            if (!record.avatarUrl) {
                return null;
            }
            return (
                <img
                    src={record.avatarUrl}
                    alt="头像"
                    className="object-fit mx-auto h-5 w-5 rounded-full"
                />
            );
        },
    },
    {
        title: "用户名",
        align: "center",
        dataIndex: "username",
    },
    {
        title: "邮箱",
        align: "center",
        dataIndex: "email",
    },
    {
        title: "真实姓名",
        align: "center",
        dataIndex: "realName",
    },
    {
        title: "状态",
        align: "center",
        dataIndex: "status",
        valueEnum: {
            1: { text: "启用", status: "Success" },
            2: { text: "禁用", status: "Default" },
        },
    },
    {
        title: "角色",
        align: "center",
        dataIndex: "roles",
        search: false,
        render: (_: React.ReactNode, record: UserItemResp) =>
            record.roles.map((role) => role.label).join(", "),
    },
    {
        title: "最后登录时间",
        align: "center",
        dataIndex: "lastLoginAt",
        valueType: "dateTime",
        search: false,
    },
    {
        title: "更新时间",
        align: "center",
        dataIndex: "updatedAt",
        valueType: "dateTime",
        search: false,
    },
    {
        title: "操作",
        key: "action",
        width: 200,
        align: "center",
        fixed: "right",
        search: false,
        render: (_dom: React.ReactNode, entity: UserItemResp, _index, action?: ActionType) => {
            const cur = useAuthStore.getState().userInfo;
            if (entity.id === cur?.id || entity.id === 1) {
                // 不能操作当前用户或管理员
                return null;
            }
            const status = entity.status === 1 ? "禁用" : "启用";
            return (
                <Space size="middle">
                    <AuthWrap code="system:user:update">
                        <UserModalForm
                            mode={"edit"}
                            initialValues={entity}
                            onSuccess={action?.reload}
                        >
                            <a>编辑</a>
                        </UserModalForm>
                    </AuthWrap>
                    <MoreButton>
                        <AuthConfirm
                            key="status"
                            code="system:user:status"
                            title={`确定要${status}此用户吗？`}
                            children={status}
                            onConfirm={async () => {
                                await userAPI.updateStatus(entity.id, entity.status === 1 ? 2 : 1);
                                void action?.reload();
                            }}
                        />
                        <AuthConfirm
                            key="password"
                            code="system:user:password"
                            title="确定要重置此用户的密码吗？"
                            children="重置密码"
                            onConfirm={async () => {
                                const randomPassword = Math.random().toString(36).slice(-8);
                                await userAPI.resetPassword(
                                    entity.id,
                                    `${entity.username}${randomPassword}`,
                                );
                                void action?.reload();
                            }}
                        />
                        <AuthConfirm
                            key="delete"
                            code="system:user:delete"
                            title="确定要删除此用户吗？"
                            className="text-red-500"
                            children="删除用户"
                            onConfirm={async () => {
                                await userAPI.delete(entity.id);
                                await action?.reload();
                            }}
                        />
                    </MoreButton>
                </Space>
            );
        },
    },
];

interface UserModalFormProps {
    initialValues?: Partial<UserItemResp>;
    mode?: "create" | "edit";
    children: React.ReactNode;
    onSuccess?: () => void;
}

const UserModalForm = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}: UserModalFormProps) => {
    const [form] = Form.useForm();
    const { data: roleOptions } = useApiQuery("system/roles/options", roleAPI.getOptions);

    return (
        <ModalForm<CreateUserDto | UpdateUserPayload>
            form={form}
            width={500}
            layout="horizontal"
            title={mode === "create" ? "创建用户" : "编辑用户"}
            trigger={children}
            labelCol={{ span: 5 }}
            modalProps={{ destroyOnHidden: true, maskClosable: false }}
            onOpenChange={(open) => {
                if (open) {
                    form.resetFields();
                    const roleIds = initialValues?.roles?.map((role) => role.value);
                    form.setFieldsValue({
                        ...initialValues,
                        roleIds,
                    });
                }
            }}
            submitter={{
                searchConfig: {
                    submitText: mode === "create" ? "创建" : "保存",
                },
            }}
            onFinish={async (values) => {
                if (mode === "create") {
                    await userAPI.create(values as CreateUserDto);
                } else if (mode === "edit" && initialValues?.id) {
                    await userAPI.update(initialValues.id, values as UpdateUserPayload);
                }
                onSuccess?.();
                form.resetFields();
                return true;
            }}
        >
            <ProFormText
                name="username"
                label="用户名"
                placeholder="请输入用户名"
                rules={[
                    { required: true, message: "请输入用户名" },
                    { min: 3, message: "至少3个字符" },
                ]}
                disabled={mode === "edit"}
            />
            <ProFormText
                name="email"
                label="邮箱"
                placeholder="请输入邮箱"
                rules={[
                    { required: true, message: "请输入邮箱" },
                    { type: "email", message: "邮箱格式不正确" },
                ]}
            />
            <ProFormText name="realName" label="真实姓名" placeholder="请输入真实姓名" />
            {mode === "create" && (
                <ProFormText.Password
                    name="password"
                    label="密码"
                    placeholder="请输入密码"
                    rules={[
                        {
                            required: true,
                            message: "请输入密码",
                        },
                        { min: 6, message: "至少6个字符" },
                    ]}
                />
            )}
            <ProFormSelect
                name="roleIds"
                label="角色"
                placeholder="请选择角色"
                options={roleOptions}
                mode="multiple"
                rules={[
                    {
                        required: true,
                        message: "请至少选择一个角色",
                    },
                ]}
            />
        </ModalForm>
    );
};
