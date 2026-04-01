import { ModalForm, ProFormText, ProFormTextArea } from "@ant-design/pro-components";
import type { ActionType, ProColumns } from "@ant-design/pro-components";
import { ProTable } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Form } from "antd";
import { Button, Space, Tag } from "antd";
import React, { useRef } from "react";

import { appMessage } from "@/api";
import { dictAPI } from "@/api/system/dict";
import type { CreateDictDto } from "@/api/types/CreateDictDto";
import type { DictItemResp } from "@/api/types/DictItemResp";
import type { UpdateDictPayload } from "@/api/types/UpdateDictPayload";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";

export const Route = createFileRoute("/system/dict")({
    component: DictPage,
});

function DictPage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <AuthWrap code="system:dict:list">
            <ProTable<DictItemResp>
                rowKey="id"
                search={false}
                scroll={{ y: "calc(100vh - 287px)" }}
                headerTitle="字典管理"
                columns={columns}
                request={dictAPI.getTableData}
                actionRef={actionRef}
                toolBarRender={() => [
                    <AuthWrap code="system:dict:create" key="create">
                        <DictModalForm
                            mode={"create"}
                            onSuccess={() => {
                                void actionRef.current?.reload();
                            }}
                        >
                            <Button type="primary">创建字典</Button>
                        </DictModalForm>
                    </AuthWrap>,
                ]}
            />
        </AuthWrap>
    );
}

const columns: ProColumns<DictItemResp>[] = [
    {
        title: "ID",
        dataIndex: "id",
        width: 48,
    },
    {
        title: "字典类型",
        align: "center",
        dataIndex: "dictType",
        ellipsis: true,
        render: (text) => <Tag color="blue">{text}</Tag>,
    },
    {
        title: "标签",
        align: "center",
        dataIndex: "label",
        ellipsis: true,
        search: {
            transform: (value) => ({ q: value }),
        },
    },
    {
        title: "值",
        align: "center",
        dataIndex: "value",
        ellipsis: true,
    },
    {
        title: "描述",
        align: "center",
        dataIndex: "description",
        ellipsis: true,
    },
    {
        title: "操作",
        align: "center",
        key: "action",
        width: 200,
        fixed: "right",
        render: (_dom: React.ReactNode, entity: DictItemResp, _index, action?: ActionType) => (
            <Space size="middle">
                <AuthWrap code="system:dict:update">
                    <DictModalForm
                        mode={"edit"}
                        initialValues={entity}
                        onSuccess={() => {
                            void action?.reload();
                        }}
                    >
                        <a>编辑</a>
                    </DictModalForm>
                </AuthWrap>
                <AuthPopconfirm
                    code="system:dict:delete"
                    title="确定要删除此字典吗？"
                    description="此操作不可撤销。"
                    onConfirm={async () => {
                        try {
                            await dictAPI.delete(entity.id);
                            appMessage.success("删除字典成功");
                            void action?.reload();
                        } catch (error) {
                            console.error("[Delete Dict Error]:", error);
                        }
                    }}
                >
                    <span className="cursor-pointer text-red-500">删除</span>
                </AuthPopconfirm>
            </Space>
        ),
    },
];

interface DictModalFormProps {
    initialValues?: Partial<DictItemResp>;
    mode?: "create" | "edit";
    children: React.ReactNode;
    onSuccess?: () => void;
}

const DictModalForm = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}: DictModalFormProps) => {
    const [form] = Form.useForm();

    return (
        <ModalForm<CreateDictDto | UpdateDictPayload>
            form={form}
            width={500}
            layout="horizontal"
            title={mode === "create" ? "创建字典" : "编辑字典"}
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
                try {
                    if (mode === "create") {
                        await dictAPI.create(values as CreateDictDto);
                        appMessage.success("创建字典成功");
                    } else if (mode === "edit" && initialValues?.id) {
                        await dictAPI.update(initialValues.id, values as UpdateDictPayload);
                        appMessage.success("更新字典成功");
                    }
                    onSuccess?.();
                    return true;
                } catch (error) {
                    console.error("[DictModalForm Submit Error]:", error);
                    return false;
                }
            }}
        >
            <ProFormText
                name="dictType"
                label="字典类型"
                placeholder="请输入字典类型（如 user_status）"
                rules={[
                    {
                        required: true,
                        message: "请输入字典类型",
                    },
                    {
                        pattern: /^[a-z_]+$/,
                        message: "字典类型只能包含小写字母和下划线",
                    },
                ]}
            />
            <ProFormText
                name="label"
                label="标签"
                placeholder="请输入显示标签（如 启用）"
                rules={[{ required: true, message: "请输入标签" }]}
            />
            <ProFormText
                name="value"
                label="值"
                placeholder="请输入值（如 1）"
                rules={[{ required: true, message: "请输入值" }]}
            />
            <ProFormTextArea name="description" label="描述" placeholder="请输入描述" />
        </ModalForm>
    );
};
