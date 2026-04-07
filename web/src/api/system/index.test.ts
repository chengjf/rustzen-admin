import { describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
    apiRequest: vi.fn(),
    apiDownload: vi.fn(),
    proTableRequest: vi.fn(),
}));

vi.mock("@/api", () => ({
    apiRequest: mocks.apiRequest,
    apiDownload: mocks.apiDownload,
    proTableRequest: mocks.proTableRequest,
}));

import { logAPI } from "./log";
import { menuAPI } from "./menu";
import { roleAPI } from "./role";
import { userAPI } from "./user";

describe("system API wrappers", () => {
    it("wraps log endpoints with the correct helpers", () => {
        void logAPI.getTableData({ action: "AUTH_LOGIN" });
        void logAPI.exportLogList();

        expect(mocks.proTableRequest).toHaveBeenCalledWith({
            url: "/api/system/logs",
            params: { action: "AUTH_LOGIN" },
        });
        expect(mocks.apiDownload).toHaveBeenCalledWith({
            url: "/api/system/logs/export",
        });
    });

    it("wraps menu endpoints with expected methods and urls", () => {
        mocks.apiRequest.mockReturnValueOnce(Promise.resolve([{ label: "菜单", value: 1 }]));

        const optionsPromise = menuAPI.getOptions();
        void menuAPI.create({ name: "菜单" } as any);
        void menuAPI.update(7, { name: "编辑" } as any);
        void menuAPI.delete(8);
        void menuAPI.getOptionsWithCode({ btn_filter: true });

        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/menus", method: "POST" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/menus/7", method: "PUT" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/menus/8", method: "DELETE" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/menus/options-with-code" }),
        );

        return expect(optionsPromise).resolves.toEqual([
            { label: "Root", value: 0 },
            { label: "菜单", value: 1 },
        ]);
    });

    it("wraps role endpoints with expected methods and urls", () => {
        void roleAPI.getTableData({ name: "admin" });
        void roleAPI.create({ name: "管理员" } as any);
        void roleAPI.update(3, { name: "更新" } as any);
        void roleAPI.delete(4);
        void roleAPI.getOptions();

        expect(mocks.proTableRequest).toHaveBeenCalledWith({
            url: "/api/system/roles",
            params: { name: "admin" },
        });
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/roles", method: "POST" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/roles/3", method: "PUT" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/roles/4", method: "DELETE" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/roles/options" }),
        );
    });

    it("wraps user endpoints with expected methods and urls", () => {
        void userAPI.getTableData({ username: "alice" });
        void userAPI.create({ username: "alice" } as any);
        void userAPI.update(10, { email: "a@test.dev" } as any);
        void userAPI.delete(11);
        void userAPI.updateStatus(12, { status: "Disabled" } as any);
        void userAPI.resetPassword(13);
        void userAPI.unlock(14);
        void userAPI.getStatusOptions();

        expect(mocks.proTableRequest).toHaveBeenCalledWith({
            url: "/api/system/users",
            params: { username: "alice" },
        });
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/users", method: "POST" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/users/10", method: "PUT" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/users/11", method: "DELETE" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/users/12/status", method: "PUT" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/users/13/password", method: "PUT" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/users/14/unlock", method: "PUT" }),
        );
        expect(mocks.apiRequest).toHaveBeenCalledWith(
            expect.objectContaining({ url: "/api/system/users/status-options" }),
        );
    });
});
