import type { UserStatus } from "@/api/types/UserStatus";

/** Default value for "enabled" status (menu, role, dict) */
export const ENABLE_DEFAULT = 1 as const;

/** Generic enable/disable options — used for menus, roles, etc. (status: 1 | 2) */
export const ENABLE_OPTIONS = [
    { label: "启用", value: ENABLE_DEFAULT },
    { label: "禁用", value: 2 },
];

/** ProTable valueEnum for enable/disable status columns */
export const ENABLE_STATUS_ENUM = {
    [ENABLE_DEFAULT]: { text: "启用", status: "Success" },
    2: { text: "禁用", status: "Default" },
} as const;

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
