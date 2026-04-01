import { create } from "zustand";

export interface TabItem {
    path: string;
    title: string;
    closable: boolean;
}

interface TabState {
    tabs: TabItem[];
    activeKey: string;
    addTab: (path: string, title: string) => void;
    removeTab: (path: string) => { tabs: TabItem[]; newActiveKey: string };
    setActiveKey: (key: string) => void;
    clearTabs: () => void;
}

const HOME_TAB: TabItem = { path: "/", title: "首页", closable: false };

export const useTabStore = create<TabState>((set, get) => ({
    tabs: [HOME_TAB],
    activeKey: "/",

    addTab: (path, title) => {
        const { tabs } = get();
        const exists = tabs.find((t) => t.path === path);
        if (!exists) {
            set({
                tabs: [...tabs, { path, title, closable: path !== "/" }],
                activeKey: path,
            });
        } else {
            set({ activeKey: path });
        }
    },

    removeTab: (path) => {
        const { tabs, activeKey } = get();
        const idx = tabs.findIndex((t) => t.path === path);
        const newTabs = tabs.filter((t) => t.path !== path);

        let newActiveKey = activeKey;
        if (activeKey === path && newTabs.length > 0) {
            // Activate the previous tab, or the next one if there's no previous
            newActiveKey = newTabs[Math.max(0, idx - 1)].path;
        }

        set({ tabs: newTabs, activeKey: newActiveKey });
        return { tabs: newTabs, newActiveKey };
    },

    setActiveKey: (key) => set({ activeKey: key }),

    clearTabs: () => set({ tabs: [HOME_TAB], activeKey: "/" }),
}));
