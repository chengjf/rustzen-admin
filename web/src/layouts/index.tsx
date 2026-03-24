import {
    BookOutlined,
    HistoryOutlined,
    MenuOutlined,
    SettingOutlined,
    TeamOutlined,
    UserOutlined,
} from "@ant-design/icons";

import { useAuthStore } from "@/stores/useAuthStore";

type AppRouter = {
    name?: string;
    icon?: React.ReactNode;
    path?: string;
    children?: AppRouter[];
};

const systemRoutes: AppRouter = {
    name: "系统管理",
    icon: <SettingOutlined />,
    path: "/system",
    children: [
        {
            path: "/system/user",
            name: "用户管理",
            icon: <UserOutlined />,
        },
        {
            path: "/system/role",
            name: "角色管理",
            icon: <TeamOutlined />,
        },
        {
            path: "/system/menu",
            name: "菜单管理",
            icon: <MenuOutlined />,
        },
        {
            path: "/system/dict",
            name: "字典管理",
            icon: <BookOutlined />,
        },
        {
            path: "/system/log",
            name: "操作日志",
            icon: <HistoryOutlined />,
        },
    ],
};

const pageRoutes: AppRouter[] = [systemRoutes];

export const getMenuData = (): AppRouter[] => {
    const { checkMenuPermissions } = useAuthStore.getState();

    const getMenuList = (menuList: AppRouter[]): AppRouter[] => {
        return menuList
            .filter((item) => {
                if (!item.path) return false;
                if (item.children) return true;
                return checkMenuPermissions(item.path);
            })
            .map((item) => {
                return {
                    ...item,
                    children: item.children ? getMenuList(item.children) : undefined,
                } as AppRouter;
            })
            .filter((item) => {
                // if none children, to hide the item
                if (item.children?.length === 0) {
                    return false;
                }
                return true;
            });
    };
    return getMenuList(pageRoutes);
};
