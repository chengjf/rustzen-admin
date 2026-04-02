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
import type { DataNode } from "antd/es/tree";
import { useRef, useMemo, useState, useCallback, useEffect, type ReactNode } from "react";

import { appMessage } from "@/api";
import { menuAPI } from "@/api/system/menu";
import { roleAPI } from "@/api/system/role";
import type { MenuTreeOption } from "@/api/types/MenuTreeOption";
import type { RoleItemResp } from "@/api/types/RoleItemResp";
import { AuthPopconfirm, AuthWrap } from "@/components/auth";
import { ENABLE_DEFAULT, ENABLE_OPTIONS, ENABLE_STATUS_ENUM } from "@/constant/options";

const { Text } = Typography;

export const Route = createFileRoute("/system/role")({
    component: RolePage,
});

// --- 辅助工具函数 ---
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

const calculateFinalIds = (checkedIds: number[], flatMap: Map<number, MenuTreeOption>) => {
    const finalSet = new Set<number>();
    checkedIds.forEach((id) => {
        let curr = flatMap.get(id);
        while (curr) {
            finalSet.add(curr.value);
            curr = curr.parentId ? flatMap.get(curr.parentId) : undefined;
        }
    });
    return Array.from(finalSet);
};

// --- 页面主组件 ---
function RolePage() {
    const actionRef = useRef<ActionType>(null);

    const handleReload = useCallback(() => {
        void actionRef.current?.reload();
    }, []);

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
                valueEnum: ENABLE_STATUS_ENUM,
            },
            {
                title: "操作",
                key: "action",
                width: 120,
                align: "center",
                fixed: "right",
                render: (_, entity) => (
                    <Space>
                        <AuthWrap code="system:role:update">
                            <RoleModalForm
                                mode="edit"
                                initialValues={entity}
                                onSuccess={handleReload}
                            >
                                <a>编辑</a>
                            </RoleModalForm>
                        </AuthWrap>
                        <AuthPopconfirm
                            title="确认删除吗？"
                            code="system:role:delete"
                            onConfirm={async () => {
                                try {
                                    await roleAPI.delete(entity.id);
                                    appMessage.success("删除成功");
                                    handleReload();
                                } catch (e) {
                                    console.error(e);
                                }
                            }}
                        >
                            <span className="text-red-500 cursor-pointer">删除</span>
                        </AuthPopconfirm>
                    </Space>
                ),
            },
        ],
        [handleReload],
    );

    const toolBarRender = useCallback(
        () => [
            <AuthWrap code="system:role:create" key="add">
                <RoleModalForm mode="create" onSuccess={handleReload}>
                    <Button type="primary">创建角色</Button>
                </RoleModalForm>
            </AuthWrap>,
        ],
        [handleReload],
    );

    return (
        <AuthWrap code="system:role:list">
            <ProTable<RoleItemResp>
                rowKey="id"
                columns={columns}
                request={roleAPI.getTableData}
                actionRef={actionRef}
                headerTitle="角色权限管理"
                toolBarRender={toolBarRender}
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

    const { filteredTree, buttonMap, flatMap } = useMemo(() => {
        const fMap = getFlatMap(menuTree);
        const bMap = new Map<number, MenuTreeOption[]>();

        // 1. 先提取按钮映射，保持 buildTree 纯净
        menuTree.forEach(function collect(node) {
            if (node.menuType === 3) {
                const btns = bMap.get(node.parentId) || [];
                bMap.set(node.parentId, [...btns, node]);
            }
            node.children?.forEach(collect);
        });

        // 2. 构建纯粹的 UI 树（排除按钮节点）
        const buildPureTree = (nodes: MenuTreeOption[]): DataNode[] => {
            return nodes
                .filter((node) => node.menuType !== 3)
                .map((node) => ({
                    title: node.label,
                    key: node.value,
                    children: node.children ? buildPureTree(node.children) : undefined,
                }));
        };

        return { filteredTree: buildPureTree(menuTree), buttonMap: bMap, flatMap: fMap };
    }, [menuTree]);

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
                    checkStrictly
                    defaultExpandAll
                    treeData={filteredTree}
                    checkedKeys={value.filter((id) => flatMap.get(id)?.menuType !== 3)}
                    onCheck={(info) => {
                        const { checked } = info as { checked: React.Key[] };
                        const checkedIds = checked.map(Number);

                        // 与右侧按钮保持一致：勾选子菜单时自动带上所有父级
                        const withParents = calculateFinalIds(checkedIds, flatMap);

                        // 保留当前已选中的按钮 IDs 不变
                        const currentButtonIds = value.filter(
                            (id) => flatMap.get(id)?.menuType === 3,
                        );

                        onChange?.([...new Set([...withParents, ...currentButtonIds])]);
                    }}
                    onSelect={(keys) => setSelectedKey(keys[0] as number)}
                    selectedKeys={selectedKey ? [selectedKey] : []}
                />
            </div>

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
                                            const next = isChecked
                                                ? value.filter((id) => id !== btn.value)
                                                : calculateFinalIds([...value, btn.value], flatMap);
                                            onChange?.(Array.from(new Set(next)));
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

// --- 表单弹窗组件 ---
interface RoleModalFormProps {
    children: ReactNode;
    initialValues?: RoleItemResp;
    mode: "create" | "edit";
    onSuccess?: () => void;
}

const RoleModalForm = ({ children, initialValues, mode, onSuccess }: RoleModalFormProps) => {
    const [form] = Form.useForm();
    const [menuTree, setMenuTree] = useState<MenuTreeOption[]>([]);
    const [loading, setLoading] = useState(false);
    const abortRef = useRef<AbortController | null>(null);

    // 组件卸载时强制中断请求
    useEffect(() => {
        return () => {
            abortRef.current?.abort();
        };
    }, []);

    const handleOpenChange = async (open: boolean) => {
        if (open) {
            abortRef.current?.abort();
            const controller = new AbortController();
            abortRef.current = controller;

            setLoading(true);
            try {
                const res = await menuAPI.getOptionsWithCode({ btn_filter: false });
                if (controller.signal.aborted) return;

                setMenuTree(res);
                if (mode === "edit" && initialValues) {
                    const fMap = getFlatMap(res);
                    const displayIds = (initialValues.menus || [])
                        .map((m) => m.value)
                        .filter((id) => fMap.has(id));
                    form.setFieldsValue({ ...initialValues, menuIds: displayIds });
                }
            } catch (e) {
                console.error("加载菜单失败", e);
            } finally {
                if (!controller.signal.aborted) {
                    setLoading(false);
                    abortRef.current = null;
                }
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
            onOpenChange={(open) => {
                void handleOpenChange(open);
            }}
            modalProps={{ destroyOnClose: true, maskClosable: false }}
            onFinish={async (values) => {
                if (loading) return false;
                try {
                    const fMap = getFlatMap(menuTree);
                    const params = {
                        ...values,
                        menuIds: calculateFinalIds(values.menuIds || [], fMap),
                    };

                    if (mode === "edit") {
                        if (!initialValues?.id) {
                            appMessage.error("数据异常，请刷新后重试");
                            console.error("edit 模式缺少 id");
                            return false;
                        }
                        await roleAPI.update(initialValues.id, params);
                    } else {
                        await roleAPI.create(params);
                    }

                    appMessage.success(mode === "edit" ? "更新成功" : "创建成功");
                    onSuccess?.();
                    return true;
                } catch (error) {
                    console.error("[RoleModalForm Submit Error]:", error);
                    return false; // 返回 false 阻止弹窗关闭
                }
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
                    <ProFormSelect
                        name="status"
                        label="状态"
                        options={ENABLE_OPTIONS}
                        initialValue={ENABLE_DEFAULT}
                        rules={[{ required: true }]}
                    />
                </Col>
            </Row>

            <Form.Item name="menuIds" label="权限配置">
                <PermissionManager menuTree={menuTree} loading={loading} />
            </Form.Item>

            <ProFormTextArea name="description" label="备注说明" />
        </ModalForm>
    );
};
