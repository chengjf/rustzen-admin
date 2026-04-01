import { UserOutlined } from "@ant-design/icons";
import type { ActionType, ProColumns } from "@ant-design/pro-components";
import { ProTable, ModalForm, ProFormSelect, ProFormText } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Form, Modal, Space, Typography, Avatar } from "antd";
import React, { useEffect, useMemo, useRef, useState, useCallback } from "react";

import { appMessage } from "@/api";
import { roleAPI } from "@/api/system/role";
import { userAPI } from "@/api/system/user";
import type { CreateUserDto } from "@/api/types/CreateUserDto";
import type { UpdateUserPayload } from "@/api/types/UpdateUserPayload";
import type { UserItemResp } from "@/api/types/UserItemResp";
import { AuthConfirm, AuthWrap } from "@/components/auth";
import { MoreButton } from "@/components/button";
import { useApiQuery } from "@/integrations/react-query";
import { useAuthStore } from "@/stores/useAuthStore";

// =============================================================================
// 1. 子组件：UserModalForm
// =============================================================================
interface UserModalFormProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    initialValues?: Partial<UserItemResp>;
    mode?: "create" | "edit";
    onSuccess?: () => void;
}

const UserModalForm = React.memo(
    ({ open, onOpenChange, initialValues, mode = "create", onSuccess }: UserModalFormProps) => {
        const [form] = Form.useForm();
        const { data: roleOptions } = useApiQuery("system/roles/options", roleAPI.getOptions);

        useEffect(() => {
            if (open) {
                form.resetFields();
                if (mode === "edit" && initialValues) {
                    // 修复：移除 any 标注，利用 TS 自动推导类型
                    // 兼容逻辑保留以应对不同版本的后端 API 定义
                    const roleIds = (initialValues.roles ?? []).map((role) => role.value);
                    form.setFieldsValue({
                        ...initialValues,
                        roleIds,
                    });
                } else {
                    form.setFieldsValue({ status: 1 });
                }
            }
        }, [open, mode, initialValues, form]);

        return (
            <ModalForm<CreateUserDto | UpdateUserPayload>
                form={form}
                open={open}
                onOpenChange={onOpenChange}
                width={500}
                layout="horizontal"
                title={mode === "create" ? "创建用户" : "编辑用户"}
                labelCol={{ span: 5 }}
                modalProps={{ destroyOnHidden: true, maskClosable: false }}
                onFinish={async (values) => {
                    try {
                        if (mode === "create") {
                            await userAPI.create(values as CreateUserDto);
                            appMessage.success("创建用户成功");
                        } else if (mode === "edit" && initialValues?.id) {
                            await userAPI.update(initialValues.id, values as UpdateUserPayload);
                            appMessage.success("更新用户成功");
                        }
                        onSuccess?.();
                        return true;
                    } catch (error) {
                        console.error("[UserModalForm Submit Error]:", error);
                        return false;
                    }
                }}
            >
                <ProFormText
                    name="username"
                    label="用户名"
                    rules={[
                        { required: true, message: "请输入用户名" },
                        { min: 3, message: "至少3个字符" },
                    ]}
                    disabled={mode === "edit"}
                />
                <ProFormText
                    name="email"
                    label="邮箱"
                    rules={[
                        { required: true, message: "请输入邮箱" },
                        { type: "email", message: "邮箱格式不正确" },
                    ]}
                />
                <ProFormText name="realName" label="真实姓名" />
                {mode === "create" && (
                    <ProFormText.Password
                        name="password"
                        label="密码"
                        rules={[
                            { required: true, message: "请输入密码" },
                            { min: 6, message: "至少6个字符" },
                        ]}
                    />
                )}
                <ProFormSelect
                    name="roleIds"
                    label="角色"
                    options={roleOptions}
                    mode="multiple"
                    rules={[{ required: true, message: "请至少选择一个角色" }]}
                />
            </ModalForm>
        );
    },
);

UserModalForm.displayName = "UserModalForm";

// =============================================================================
// 2. 主页面：UserPage
// =============================================================================
export const Route = createFileRoute("/system/user")({
    component: UserPage,
});

function UserPage() {
    const actionRef = useRef<ActionType>(null);
    const userInfo = useAuthStore((state) => state.userInfo);

    const [passwordModal, setPasswordModal] = useState({ open: false, password: "" });
    const [modalState, setModalState] = useState<{
        open: boolean;
        mode: "create" | "edit";
        record?: Partial<UserItemResp>;
    }>({ open: false, mode: "create" });

    const closePasswordModal = useCallback(() => {
        setPasswordModal({ open: false, password: "" });
    }, []);

    const handleOpenChange = useCallback((open: boolean) => {
        setModalState((prev) => ({ ...prev, open }));
    }, []);

    const handleModalSuccess = useCallback(() => {
        setModalState((prev) => ({ ...prev, open: false }));
        void actionRef.current?.reload();
    }, []);

    const handleEdit = useCallback((record: UserItemResp) => {
        setModalState({ open: true, mode: "edit", record });
    }, []);

    const handleCreate = useCallback(() => {
        setModalState({ open: true, mode: "create" });
    }, []);

    const columns: ProColumns<UserItemResp>[] = useMemo(
        () => [
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
                render: (url) => (
                    <Avatar src={url as string} icon={<UserOutlined />} className="mx-auto" />
                ),
            },
            { title: "用户名", align: "center", dataIndex: "username" },
            { title: "真实姓名", align: "center", dataIndex: "realName" },
            {
                title: "状态",
                align: "center",
                dataIndex: "status",
                valueEnum: {
                    1: { text: "启用", status: "Success" },
                    2: { text: "禁用", status: "Default" },
                },
            },
            { title: "邮箱", align: "center", dataIndex: "email" },
            {
                title: "角色",
                align: "center",
                dataIndex: "roles",
                search: false,
                render: (_, record) => (record.roles ?? []).map((role) => role.label).join(", "),
            },
            {
                title: "最后登录时间",
                align: "center",
                dataIndex: "lastLoginAt",
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
                render: (_, entity) => {
                    if (entity.id === userInfo?.id || entity.id === 1) return null;
                    const isEnable = entity.status === 1;
                    const statusText = isEnable ? "禁用" : "启用";

                    return (
                        <Space size="middle">
                            <AuthWrap code="system:user:update">
                                <a onClick={() => handleEdit(entity)}>编辑</a>
                            </AuthWrap>
                            <MoreButton>
                                <AuthConfirm
                                    key="status"
                                    code="system:user:status"
                                    title={`确定要${statusText}此用户吗？`}
                                    onConfirm={async () => {
                                        await userAPI.updateStatus(entity.id, {
                                            status: isEnable ? 2 : 1,
                                        });
                                        appMessage.success(
                                            statusText === "启用" ? "已启用用户" : "已禁用用户",
                                        );
                                        void actionRef.current?.reload();
                                    }}
                                >
                                    {statusText}
                                </AuthConfirm>
                                <AuthConfirm
                                    key="password"
                                    code="system:user:password"
                                    title="确定要重置此用户的密码吗？"
                                    onConfirm={async () => {
                                        const res = await userAPI.resetPassword(entity.id);
                                        setPasswordModal({ open: true, password: res.password });
                                        // 修复：移除重置密码后无意义的 reload
                                    }}
                                >
                                    重置密码
                                </AuthConfirm>
                                <AuthConfirm
                                    key="delete"
                                    code="system:user:delete"
                                    title="确定要删除此用户吗？"
                                    className="text-red-500"
                                    okButtonProps={{ danger: true }}
                                    onConfirm={async () => {
                                        await userAPI.delete(entity.id);
                                        appMessage.success("删除用户成功");
                                        void actionRef.current?.reload();
                                    }}
                                >
                                    删除用户
                                </AuthConfirm>
                            </MoreButton>
                        </Space>
                    );
                },
            },
        ],
        [userInfo, handleEdit],
    );

    const toolBarRender = useCallback(
        () => [
            <AuthWrap code="system:user:create" key="create">
                <Button type="primary" onClick={handleCreate}>
                    创建用户
                </Button>
            </AuthWrap>,
        ],
        [handleCreate],
    );

    return (
        <>
            <AuthWrap code="system:user:list">
                <ProTable<UserItemResp>
                    rowKey="id"
                    scroll={{ y: "calc(100vh - 383px)" }}
                    headerTitle="用户列表"
                    columns={columns}
                    request={userAPI.getTableData}
                    actionRef={actionRef}
                    search={{ span: 6 }}
                    toolBarRender={toolBarRender}
                />
            </AuthWrap>

            <UserModalForm
                open={modalState.open}
                mode={modalState.mode}
                initialValues={modalState.record}
                onOpenChange={handleOpenChange}
                onSuccess={handleModalSuccess}
            />

            <Modal
                title="密码重置成功"
                open={passwordModal.open}
                onOk={closePasswordModal}
                onCancel={closePasswordModal}
                okText="关闭"
                cancelButtonProps={{ style: { display: "none" } }}
            >
                <p>该用户的临时新密码如下，请及时通知用户修改：</p>
                <Typography.Text
                    strong
                    copyable={{ text: passwordModal.password }}
                    style={{
                        fontSize: 24,
                        display: "block",
                        textAlign: "center",
                        margin: "16px 0",
                    }}
                >
                    {passwordModal.password}
                </Typography.Text>
            </Modal>
        </>
    );
}
