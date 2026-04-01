import { CloseOutlined, ReloadOutlined, CloseCircleOutlined } from "@ant-design/icons";
import { useNavigate } from "@tanstack/react-router";
import { Dropdown } from "antd";
import type { MenuProps } from "antd";
import { useCallback, useRef } from "react";

import { useTabStore, type TabItem } from "@/stores/useTabStore";

interface TabBarProps {
    onReload?: () => void;
}

export const TabBar = ({ onReload }: TabBarProps) => {
    const navigate = useNavigate();
    const { tabs, activeKey, removeTab, setActiveKey, clearTabs } = useTabStore();
    const rightClickTab = useRef<string>("");

    const handleTabClick = useCallback(
        (path: string) => {
            setActiveKey(path);
            void navigate({ to: path as any });
        },
        [navigate, setActiveKey],
    );

    const handleClose = useCallback(
        (e: React.MouseEvent, path: string) => {
            e.stopPropagation();
            const { newActiveKey } = removeTab(path);
            if (activeKey === path) {
                void navigate({ to: newActiveKey as any });
            }
        },
        [removeTab, activeKey, navigate],
    );

    const handleCloseOthers = useCallback(() => {
        const path = rightClickTab.current;
        const { tabs: currentTabs } = useTabStore.getState();
        // Keep home + the right-clicked tab
        currentTabs
            .filter((t) => t.closable && t.path !== path)
            .forEach((t) => removeTab(t.path));
        setActiveKey(path);
        void navigate({ to: path as any });
    }, [removeTab, setActiveKey, navigate]);

    const handleCloseAll = useCallback(() => {
        clearTabs();
        void navigate({ to: "/" });
    }, [clearTabs, navigate]);

    const contextMenuItems = (tab: TabItem): MenuProps["items"] => [
        {
            key: "reload",
            icon: <ReloadOutlined />,
            label: "刷新当前",
            disabled: tab.path !== activeKey,
            onClick: () => onReload?.(),
        },
        { type: "divider" },
        {
            key: "closeOthers",
            icon: <CloseCircleOutlined />,
            label: "关闭其他",
            disabled: tabs.filter((t) => t.closable).length <= 1,
            onClick: handleCloseOthers,
        },
        {
            key: "closeAll",
            icon: <CloseCircleOutlined />,
            label: "关闭全部",
            disabled: tabs.filter((t) => t.closable).length === 0,
            onClick: handleCloseAll,
        },
    ];

    return (
        <div className="tab-bar">
            <div className="tab-bar-inner">
                {tabs.map((tab) => {
                    const isActive = tab.path === activeKey;
                    return (
                        <Dropdown
                            key={tab.path}
                            menu={{ items: contextMenuItems(tab) }}
                            trigger={["contextMenu"]}
                            onOpenChange={(open) => {
                                if (open) rightClickTab.current = tab.path;
                            }}
                        >
                            <div
                                className={`tab-item ${isActive ? "tab-item--active" : ""}`}
                                onClick={() => handleTabClick(tab.path)}
                            >
                                <span className="tab-item__label">{tab.title}</span>
                                {tab.closable && (
                                    <span
                                        className="tab-item__close"
                                        onClick={(e) => handleClose(e, tab.path)}
                                    >
                                        <CloseOutlined />
                                    </span>
                                )}
                            </div>
                        </Dropdown>
                    );
                })}
            </div>

            <style>{`
                .tab-bar {
                    background: #f5f5f5;
                    border-bottom: 1px solid #e8e8e8;
                    padding: 6px 12px 0;
                    overflow-x: auto;
                    overflow-y: hidden;
                    white-space: nowrap;
                    scrollbar-width: none;
                }
                .tab-bar::-webkit-scrollbar {
                    display: none;
                }
                .tab-bar-inner {
                    display: inline-flex;
                    gap: 4px;
                    align-items: flex-end;
                }
                .tab-item {
                    display: inline-flex;
                    align-items: center;
                    gap: 6px;
                    padding: 5px 14px;
                    background: #e2e2e2;
                    border-radius: 6px 6px 0 0;
                    cursor: pointer;
                    font-size: 13px;
                    color: #666;
                    border: 1px solid transparent;
                    border-bottom: none;
                    transition: background 0.15s, color 0.15s;
                    user-select: none;
                    max-width: 160px;
                    position: relative;
                    top: 1px;
                }
                .tab-item:hover {
                    background: #d5d5d5;
                    color: #333;
                }
                .tab-item--active {
                    background: #fff;
                    color: #1677ff;
                    border-color: #e8e8e8;
                    font-weight: 500;
                }
                .tab-item--active:hover {
                    background: #fff;
                    color: #1677ff;
                }
                .tab-item__label {
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }
                .tab-item__close {
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    width: 16px;
                    height: 16px;
                    border-radius: 50%;
                    font-size: 10px;
                    color: #999;
                    flex-shrink: 0;
                    transition: background 0.15s, color 0.15s;
                }
                .tab-item__close:hover {
                    background: #bbb;
                    color: #fff;
                }
                .tab-item--active .tab-item__close:hover {
                    background: #1677ff22;
                    color: #1677ff;
                }
            `}</style>
        </div>
    );
};
