import { DashboardOutlined, DownOutlined, KeyOutlined, LogoutOutlined, UserOutlined } from "@ant-design/icons";
import { ProLayout } from "@ant-design/pro-components";
import { Link, useLocation, useRouter } from "@tanstack/react-router";
import type { MenuProps } from "antd";
import { Dropdown } from "antd";

import { appMessage } from "@/api";
import { authAPI } from "@/api/auth";
import { ChangePasswordModal } from "@/components/user/ChangePasswordModal";
import { UserProfileModal } from "@/components/user/index";
import { getMenuData } from "@/layouts";
import { useAuthStore } from "@/stores/useAuthStore";

interface BasicLayoutProps {
    children: React.ReactNode;
    hidden?: boolean;
}

export const BasicLayout = ({ children, hidden = false }: BasicLayoutProps) => {
    const { userInfo } = useAuthStore();
    const router = useRouter();
    const currentPath = useLocation().pathname;

    // If hidden, return children
    if (hidden) {
        return children;
    }

    // User dropdown menu items
    const userMenuItems: MenuProps["items"] = [
        {
            key: "profile",
            icon: <UserOutlined />,
            label: <UserProfileModal />,
        },
        {
            key: "changePassword",
            icon: <KeyOutlined />,
            label: <ChangePasswordModal />,
        },
        {
            type: "divider",
        },
        {
            key: "logout",
            icon: <LogoutOutlined />,
            label: "退出登录",
            onClick: async () => {
                await authAPI.logout();
                useAuthStore.getState().clearAuth();
                appMessage.success("退出登录成功");
                void router.navigate({ to: "/login" });
                return true;
            },
        },
    ];
    return (
        <ProLayout
            title="Rustzen Admin"
            logo="/rustzen.png"
            location={{ pathname: currentPath }}
            layout="mix"
            onMenuHeaderClick={() => router.navigate({ to: "/" })}
            menuItemRender={(item, dom) => (
                <Link to={item.path || "/"} className="block">
                    {dom}
                </Link>
            )}
            route={{
                path: "/",
                children: [
                    {
                        path: "/",
                        name: "首页",
                        icon: <DashboardOutlined />,
                    },
                    ...getMenuData(),
                ],
            }}
            avatarProps={{
                src: userInfo?.avatarUrl,
                size: "small",
                title: null,
                render: (_props, dom) => {
                    return (
                        <Dropdown menu={{ items: userMenuItems }}>
                            <div className="flex items-center gap-2 px-3 py-1.5 rounded-full cursor-pointer hover:bg-gray-100 transition-colors">
                                {dom}
                                <span className="text-sm font-medium text-gray-700">
                                    {userInfo?.realName || userInfo?.username}
                                </span>
                                <DownOutlined className="text-xs text-gray-500" />
                            </div>
                        </Dropdown>
                    );
                },
            }}
        >
            {children}
        </ProLayout>
    );
};
