// ==================== 菜单管理 ====================
declare namespace Menu {
    // 菜单状态枚举
    enum Status {
        Normal = 1,
        Disabled = 2,
    }

    // 菜单类型枚举
    enum MenuType {
        Directory = 1,
        Menu = 2,
        Button = 3,
    }

    // 菜单基本信息 - 简化版本
    interface Item {
        id: number;
        parentId: number;
        name: string;
        code: string;
        menuType: MenuType;
        sortOrder: number;
        status: Status;
        isSystem: boolean;
        createdAt: string;
        updatedAt: string;
        children?: Item[] | null;
    }

    // 查询参数
    interface QueryParams {
        name?: string;
        code?: string;
        status?: Status;
        menuType?: MenuType;
    }

    // 创建菜单请求
    interface CreateAndUpdateRequest {
        parentId: number;
        name: string;
        code: string;
        menuType: MenuType;
        sortOrder: number;
        status: Status;
    }

    interface OptionsWithCodeQuery {
        q?: string;
        limit?: number;
        btn_filter?: boolean;
    }
}
