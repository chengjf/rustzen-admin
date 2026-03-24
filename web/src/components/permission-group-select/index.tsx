import { Checkbox, Col, Divider, Input, Row, Tag, Typography } from "antd";
import React, { useMemo, useState } from "react";

const { Text } = Typography;

interface PermissionGroupSelectProps {
    value?: number[];
    onChange?: (value: number[]) => void;
    options: Api.MenuOptionItem[];
    disabled?: boolean;
}

interface GroupedPermissions {
    [key: string]: {
        label: string;
        items: Api.MenuOptionItem[];
    };
}

const menuTypeLabels: Record<number, string> = {
    1: "目录",
    2: "菜单",
    3: "按钮",
};

export const PermissionGroupSelect: React.FC<PermissionGroupSelectProps> = ({
    value = [],
    onChange,
    options,
    disabled = false,
}) => {
    const [searchText, setSearchText] = useState("");

    // 按模块分组权限
    const groupedPermissions = useMemo(() => {
        const groups: GroupedPermissions = {};

        options.forEach((item) => {
            // 从 code 中提取模块名，如 system:role:list -> system:role
            const codeParts = item.code.split(":");
            const moduleKey = codeParts.slice(0, -1).join(":") || "other";
            const moduleName = moduleKey || "其他";

            if (!groups[moduleKey]) {
                groups[moduleKey] = {
                    label: moduleName,
                    items: [],
                };
            }
            groups[moduleKey].items.push(item);
        });

        // 按模块名排序
        return Object.entries(groups)
            .sort(([a], [b]) => a.localeCompare(b))
            .map(([key, group]) => ({
                key,
                ...group,
            }));
    }, [options]);

    // 过滤分组
    const filteredGroups = useMemo(() => {
        if (!searchText) return groupedPermissions;

        const lowerSearch = searchText.toLowerCase();
        return groupedPermissions
            .map((group) => ({
                ...group,
                items: group.items.filter(
                    (item) =>
                        item.label.toLowerCase().includes(lowerSearch) ||
                        item.code.toLowerCase().includes(lowerSearch)
                ),
            }))
            .filter((group) => group.items.length > 0);
    }, [groupedPermissions, searchText]);

    // 处理单个权限选择
    const handleItemChange = (itemId: number, checked: boolean) => {
        if (!onChange) return;

        if (checked) {
            onChange([...value, itemId]);
        } else {
            onChange(value.filter((id) => id !== itemId));
        }
    };

    // 处理组内全选
    const handleGroupCheckAll = (groupKey: string, checked: boolean) => {
        if (!onChange) return;

        const group = groupedPermissions.find((g) => g.key === groupKey);
        if (!group) return;

        const groupItemIds = group.items.map((item) => item.value);

        if (checked) {
            // 添加组内所有未选中的权限
            const newValue = [...new Set([...value, ...groupItemIds])];
            onChange(newValue);
        } else {
            // 移除组内所有权限
            onChange(value.filter((id) => !groupItemIds.includes(id)));
        }
    };

    // 检查组是否全选
    const isGroupChecked = (groupKey: string) => {
        const group = groupedPermissions.find((g) => g.key === groupKey);
        if (!group) return false;

        const groupItemIds = group.items.map((item) => item.value);
        return groupItemIds.every((id) => value.includes(id));
    };

    // 检查组是否部分选中
    const isGroupIndeterminate = (groupKey: string) => {
        const group = groupedPermissions.find((g) => g.key === groupKey);
        if (!group) return false;

        const groupItemIds = group.items.map((item) => item.value);
        const checkedCount = groupItemIds.filter((id) => value.includes(id)).length;

        return checkedCount > 0 && checkedCount < groupItemIds.length;
    };

    return (
        <div
            style={{
                border: "1px solid #d9d9d9",
                borderRadius: "6px",
                padding: "12px",
                maxHeight: "400px",
                overflow: "auto",
            }}
        >
            <Input.Search
                placeholder="搜索权限名称或代码"
                value={searchText}
                onChange={(e) => setSearchText(e.target.value)}
                style={{ marginBottom: "12px" }}
                allowClear
            />

            {filteredGroups.length === 0 ? (
                <Text type="secondary">暂无权限数据</Text>
            ) : (
                filteredGroups.map((group) => (
                    <div key={group.key} style={{ marginBottom: "16px" }}>
                        <div
                            style={{
                                display: "flex",
                                alignItems: "center",
                                marginBottom: "8px",
                            }}
                        >
                            <Checkbox
                                checked={isGroupChecked(group.key)}
                                indeterminate={isGroupIndeterminate(group.key)}
                                onChange={(e) => handleGroupCheckAll(group.key, e.target.checked)}
                                disabled={disabled}
                            />
                            <Tag color="blue" style={{ marginLeft: "8px" }}>
                                {group.label}
                            </Tag>
                            <Text type="secondary" style={{ fontSize: "12px" }}>
                                ({group.items.length} 项)
                            </Text>
                        </div>

                        <Row gutter={[8, 8]} style={{ marginLeft: "24px" }}>
                            {group.items.map((item) => (
                                <Col key={item.value} span={8}>
                                    <Checkbox
                                        checked={value.includes(item.value)}
                                        onChange={(e) =>
                                            handleItemChange(item.value, e.target.checked)
                                        }
                                        disabled={disabled}
                                    >
                                        <span
                                            style={{
                                                fontSize: "12px",
                                                color: "#666",
                                            }}
                                        >
                                            {item.label}
                                        </span>
                                        <br />
                                        <Text
                                            type="secondary"
                                            style={{
                                                fontSize: "11px",
                                                fontFamily: "monospace",
                                            }}
                                        >
                                            {item.code}
                                        </Text>
                                    </Checkbox>
                                </Col>
                            ))}
                        </Row>

                        <Divider style={{ margin: "12px 0" }} />
                    </div>
                ))
            )}

            <div
                style={{
                    marginTop: "8px",
                    padding: "8px",
                    backgroundColor: "#f5f5f5",
                    borderRadius: "4px",
                }}
            >
                <Text type="secondary" style={{ fontSize: "12px" }}>
                    已选择 {value.length} 项权限
                </Text>
            </div>
        </div>
    );
};
