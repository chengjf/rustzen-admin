import type { UserStatus } from "@/api/types/UserStatus";

/** Generic enable/disable options — used for menus, roles, etc. (status: 1 | 2) */
export const ENABLE_OPTIONS = [
    { label: "启用", value: 1 },
    { label: "禁用", value: 2 },
];

/** Full user status options — reflects all four UserStatus variants */
export const USER_STATUS_OPTIONS: { label: string; value: UserStatus }[] = [
    { label: "正常", value: "Normal" },
    { label: "禁用", value: "Disabled" },
    { label: "待审核", value: "Pending" },
    { label: "锁定", value: "Locked" },
];

export const MENU_TYPE_OPTIONS = [
    { label: "目录", value: 1 },
    { label: "菜单", value: 2 },
    { label: "按钮", value: 3 },
];
