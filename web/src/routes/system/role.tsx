import {
    ModalForm,
    ProTable,
    ProFormText,
    ProFormTextArea,
    ProFormSelect,
    ProFormDigit,
    type ActionType,
    type ProColumns,
} from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Form, Space, Tree, Typography, Checkbox, Card, Empty, Row, Col, Spin, Button } from "antd";
import { useRef, useMemo, useState, type ReactNode } from "react";

import { menuAPI } from "@/api/system/menu";
import { roleAPI } from "@/api/system/role";
import type { RoleItemResp } from "@/api/types/RoleItemResp";
import type { MenuTreeOption } from "@/api/types/MenuTreeOption";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";
import { ENABLE_OPTIONS } from "@/constant/options";

const { Text } = Typography;

export const Route = createFileRoute("/system/role")({
    component: RolePage,
});

// --- 核心工具函数 ---
const getFlatMap = (nodes: MenuTreeOption[]) => {
    const map = new Map<number, MenuTreeOption>();
    const traverse = (data: MenuTreeOption[]) => {
        data.forEach((node) => {
            map.set(node.value, node);
            if (node.children) traverse(node.children);
        });
    };
    traverse(nodes);
    return map;
};

// 提交前：根据选中的菜单/按钮，向上追溯所有目录 ID
const calculateFinalIds = (checkedIds: number[], flatMap: Map<number, MenuTreeOption>) => {
    const finalSet = new Set<number>();
    checkedIds.forEach((id) => {
        let curr = flatMap.get(id);
        while (curr) {
            finalSet.add(curr.value);
            curr = curr.parentId ? flatMap.get(curr.parentId) : undefined; // 向上追溯
        }
    });
    return Array.from(finalSet);
};

function RolePage() {
    const actionRef = useRef<ActionType>(null);
    const columns: ProColumns<RoleItemResp>[] = useMemo(
        () => [
            { title: "ID", dataIndex: "id", width: 60, align: "center", search: false },
            { title: "角色名称", dataIndex: "name", width: 150, align: "center" },
            { title: "角色编码", dataIndex: "code", width: 150, align: "center" },
            { title: "排序", dataIndex: "sortOrder", width: 80, align: "center", search: false },
            {
                title: "状态",
                dataIndex: "status",
                width: 100,
                align: "center",
                valueEnum: {
                    1: { text: "启用", status: "Success" },
                    2: { text: "禁用", status: "Default" },
                },
            },
            {
                title: "操作",
                key: "action",
                width: 120,
                align: "center",
                fixed: "right",
                render: (_, entity, __, action) => (
                    <Space>
                        <AuthWrap code="system:role:update">
                            <RoleModalForm
                                mode="edit"
                                initialValues={entity}
                                onSuccess={() => action?.reload()}
                            >
                                <a>编辑</a>
                            </RoleModalForm>
                        </AuthWrap>
                        <AuthPopconfirm
                            title="确认删除吗？"
                            code="system:role:delete"
                            onConfirm={async () => {
                                await roleAPI.delete(entity.id);
                                void action?.reload();
                            }}
                        >
                            <span className="text-red-500 cursor-pointer">删除</span>
                        </AuthPopconfirm>
                    </Space>
                ),
            },
        ],
        [],
    );

    return (
        <AuthWrap code="system:role:list">
            <ProTable<RoleItemResp>
                rowKey="id"
                columns={columns}
                request={roleAPI.getTableData}
                actionRef={actionRef}
                headerTitle="角色权限管理"
                toolBarRender={() => [
                    <AuthWrap code="system:role:create" key="add">
                        <RoleModalForm mode="create" onSuccess={() => actionRef.current?.reload()}>
                            <Button type="primary">创建角色</Button>
                        </RoleModalForm>
                    </AuthWrap>,
                ]}
            />
        </AuthWrap>
    );
}

// --- 权限分配组件 ---
interface PermissionManagerProps {
    value?: number[];
    onChange?: (ids: number[]) => void;
    menuTree: MenuTreeOption[];
    loading: boolean;
}

const PermissionManager = ({ value = [], onChange, menuTree, loading }: PermissionManagerProps) => {
    const [selectedKey, setSelectedKey] = useState<number | null>(null);

    // 构建树与索引
    const { filteredTree, buttonMap, flatMap } = useMemo(() => {
        const bMap = new Map<number, MenuTreeOption[]>();
        const fMap = getFlatMap(menuTree);

        const buildTree = (nodes: MenuTreeOption[]): object[] => {
            return nodes
                .map((node) => {
                    if (node.menuType === 3) {
                        const btns = bMap.get(node.parentId) || [];
                        bMap.set(node.parentId, [...btns, node]);
                        return null;
                    }
                    return {
                        title: node.label,
                        key: node.value,
                        menuType: node.menuType,
                        children: node.children
                            ? buildTree(node.children).filter(Boolean)
                            : undefined,
                    };
                })
                .filter(Boolean) as object[];
        };

        return { filteredTree: buildTree(menuTree), buttonMap: bMap, flatMap: fMap };
    }, [menuTree]);

    // 获取当前选中菜单的按钮
    const currentButtons = useMemo(() => {
        return selectedKey ? buttonMap.get(selectedKey) || [] : [];
    }, [selectedKey, buttonMap]);

    if (loading)
        return (
            <div style={{ padding: 40, textAlign: "center" }}>
                <Spin tip="加载菜单中..." />
            </div>
        );

    return (
        <div
            style={{
                display: "flex",
                border: "1px solid #d9d9d9",
                borderRadius: 8,
                height: 450,
                overflow: "hidden",
            }}
        >
            {/* 左侧：菜单树 */}
            <div
                style={{
                    width: 300,
                    borderRight: "1px solid #d9d9d9",
                    padding: 12,
                    overflow: "auto",
                    background: "#fafafa",
                }}
            >
                <Tree
                    checkable
                    checkStrictly // 严格受控，手动处理父子逻辑更可靠
                    defaultExpandAll
                    treeData={filteredTree}
                    checkedKeys={value.filter((id: number) => {
                        const node = flatMap.get(id);
                        return node ? node.menuType !== 3 : false;
                    })}
                    onCheck={(checkedInfo: any) => {
                        const checkedKeys = checkedInfo.checked as number[];
                        const buttonIds = value.filter(
                            (id: number) => flatMap.get(id)?.menuType === 3,
                        );
                        onChange?.([...checkedKeys, ...buttonIds]);
                    }}
                    onSelect={(keys) => setSelectedKey(keys[0] as number)}
                    selectedKeys={selectedKey ? [selectedKey] : []}
                />
            </div>

            {/* 右侧：按钮列表 */}
            <div style={{ flex: 1, display: "flex", flexDirection: "column", background: "#fff" }}>
                <div
                    style={{
                        padding: "12px 16px",
                        background: "#f5f5f5",
                        borderBottom: "1px solid #d9d9d9",
                    }}
                >
                    <Text strong>
                        功能配置：{selectedKey ? flatMap.get(selectedKey)?.label : "请选择左侧菜单"}
                    </Text>
                </div>
                <div style={{ flex: 1, padding: 16, overflow: "auto" }}>
                    {currentButtons.length > 0 ? (
                        <Row gutter={[12, 12]}>
                            {currentButtons.map((btn) => (
                                <Col span={12} key={btn.value}>
                                    <Card
                                        size="small"
                                        hoverable
                                        onClick={() => {
                                            const isChecked = value.includes(btn.value);
                                            let next: number[];
                                            if (isChecked) {
                                                next = value.filter((id) => id !== btn.value);
                                            } else {
                                                // 选中按钮，递归向上追溯所有祖先菜单确保都被选中
                                                next = Array.from(
                                                    new Set(calculateFinalIds([...value, btn.value], flatMap)),
                                                );
                                            }
                                            onChange?.(next);
                                        }}
                                        style={{
                                            cursor: "pointer",
                                            borderColor: value.includes(btn.value)
                                                ? "#1677ff"
                                                : "#f0f0f0",
                                            background: value.includes(btn.value)
                                                ? "#e6f4ff"
                                                : "#fff",
                                        }}
                                    >
                                        <Checkbox
                                            checked={value.includes(btn.value)}
                                            style={{ pointerEvents: "none" }}
                                        >
                                            {btn.label}
                                        </Checkbox>
                                    </Card>
                                </Col>
                            ))}
                        </Row>
                    ) : (
                        <Empty
                            image={Empty.PRESENTED_IMAGE_SIMPLE}
                            description={selectedKey ? "该菜单下无功能按钮" : "请在左侧选择菜单"}
                        />
                    )}
                </div>
            </div>
        </div>
    );
};

interface RoleModalFormProps {
    children: ReactNode;
    initialValues?: RoleItemResp;
    mode: "create" | "edit";
    onSuccess?: () => void;
}

// --- ModalForm 组件 ---
const RoleModalForm = ({ children, initialValues, mode, onSuccess }: RoleModalFormProps) => {
    const [form] = Form.useForm();
    const [menuTree, setMenuTree] = useState<MenuTreeOption[]>([]);
    const [loading, setLoading] = useState(false);
    const abortRef = useRef<AbortController | null>(null);

    const handleOpen = async (open: boolean) => {
        if (open) {
            // 取消上一次未完成的请求，防止快速开关时的状态泄漏
            abortRef.current?.abort();
            const controller = new AbortController();
            abortRef.current = controller;

            setLoading(true);
            try {
                const res = await menuAPI.getOptionsWithCode({ btn_filter: false });

                // 请求已被取消则忽略结果
                if (controller.signal.aborted) return;

                setMenuTree(res);

                if (mode === "edit" && initialValues) {
                    const fMap = getFlatMap(res);
                    const rawIds = initialValues.menus?.map((m: { value: number }) => m.value) ?? [];
                    const displayIds = rawIds.filter((id: number) => fMap.has(id));
                    form.setFieldsValue({ ...initialValues, menuIds: displayIds });
                }
            } finally {
                if (!controller.signal.aborted) setLoading(false);
            }
        } else {
            abortRef.current?.abort();
            form.resetFields();
        }
    };

    return (
        <ModalForm
            form={form}
            title={mode === "edit" ? "编辑角色" : "新建角色"}
            trigger={children}
            width={850}
            layout="vertical"
            onOpenChange={handleOpen}
            modalProps={{
                destroyOnClose: true,
                maskClosable: false,
            }}
            onFinish={async (values) => {
                if (loading || menuTree.length === 0) return false;

                const fMap = getFlatMap(menuTree);
                const finalMenuIds = calculateFinalIds(values.menuIds || [], fMap);
                const params = { ...values, menuIds: finalMenuIds };

                if (mode === "edit") {
                    await roleAPI.update(initialValues!.id, params);
                } else {
                    await roleAPI.create(params);
                }
                onSuccess?.();
                return true;
            }}
        >
            <Row gutter={24}>
                <Col span={6}>
                    <ProFormText name="name" label="角色名称" rules={[{ required: true }]} />
                </Col>
                <Col span={6}>
                    <ProFormText name="code" label="角色编码" rules={[{ required: true }]} />
                </Col>
                <Col span={6}>
                    <ProFormDigit name="sortOrder" label="排序" initialValue={0} />
                </Col>
                <Col span={6}>
                    <ProFormSelect name="status" label="状态" options={ENABLE_OPTIONS} initialValue={1} rules={[{ required: true }]} />
                </Col>
            </Row>

            {/* 这里增加一个判断：只有 menuTree 有数据或者正在加载时才渲染权限组件
         防止 PermissionManager 因为空数据产生错误的初始计算
      */}
            <Form.Item name="menuIds" label="权限配置">
                <PermissionManager menuTree={menuTree} loading={loading} />
            </Form.Item>

            <ProFormTextArea name="description" label="备注说明" />
        </ModalForm>
    );
};
