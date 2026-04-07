import { cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const mocks = vi.hoisted(() => ({
    fetchMock: vi.fn(),
    messageError: vi.fn(),
    messageSuccess: vi.fn(),
    modalConfirm: vi.fn(),
    navigate: vi.fn(() => Promise.resolve()),
}));

vi.mock("antd", () => ({
    App: {
        useApp: () => ({
            message: {
                error: mocks.messageError,
                success: mocks.messageSuccess,
            },
            modal: {
                confirm: mocks.modalConfirm,
            },
            notification: {},
        }),
    },
}));

vi.mock("@/router", () => ({
    router: {
        navigate: mocks.navigate,
    },
}));

import { MessageContent, apiDownload, apiRequest, proTableRequest } from "@/api";

describe("apiRequest error handling", () => {
    beforeEach(() => {
        render(<MessageContent />);
        useAuthStore.setState({ token: null, userInfo: null });
        vi.stubGlobal("fetch", mocks.fetchMock);
        vi.spyOn(console, "error").mockImplementation(() => {});
        vi.spyOn(console, "debug").mockImplementation(() => {});
        vi.spyOn(console, "warn").mockImplementation(() => {});
    });

    afterEach(async () => {
        cleanup();
        useAuthStore.setState({ token: null, userInfo: null });
        vi.unstubAllGlobals();
        vi.clearAllMocks();
        await Promise.resolve();
    });

    it("clears auth and redirects to login on 401", async () => {
        useAuthStore.setState({ token: "expired-token", userInfo: mockUserInfo });
        mocks.fetchMock.mockResolvedValue(
            new Response(JSON.stringify({ message: "会话已过期" }), {
                status: 401,
                headers: { "Content-Type": "application/json" },
            }),
        );

        await expect(apiRequest({ url: "/api/auth/me" })).rejects.toBeInstanceOf(Response);

        expect(useAuthStore.getState().token).toBeNull();
        expect(useAuthStore.getState().userInfo).toBeNull();
        expect(mocks.messageError).toHaveBeenCalledWith("会话已过期");
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/login", replace: true });
    });

    it("falls back to the default 401 message when the body is not json", async () => {
        useAuthStore.setState({ token: "expired-token", userInfo: mockUserInfo });
        mocks.fetchMock.mockResolvedValue(
            new Response("expired", {
                status: 401,
                statusText: "Unauthorized",
            }),
        );

        await expect(apiRequest({ url: "/api/auth/fallback" })).rejects.toBeInstanceOf(Response);

        expect(mocks.messageError).toHaveBeenCalledWith("会话过期，请重新登录");
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/login", replace: true });
    });

    it("supports silent 401 handling without showing a toast", async () => {
        useAuthStore.setState({ token: "expired-token", userInfo: mockUserInfo });
        mocks.fetchMock.mockResolvedValue(
            new Response(JSON.stringify({ message: "会话已过期" }), {
                status: 401,
                headers: { "Content-Type": "application/json" },
            }),
        );

        await expect(apiRequest({ url: "/api/auth/silent", silent: true })).rejects.toBeInstanceOf(
            Response,
        );

        expect(mocks.messageError).not.toHaveBeenCalled();
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/login", replace: true });
    });

    it("handles concurrent 401 responses with a single redirect and toast", async () => {
        useAuthStore.setState({ token: "expired-token", userInfo: mockUserInfo });
        mocks.fetchMock
            .mockResolvedValueOnce(
                new Response(JSON.stringify({ message: "会话已过期" }), {
                    status: 401,
                    headers: { "Content-Type": "application/json" },
                }),
            )
            .mockResolvedValueOnce(
                new Response(JSON.stringify({ message: "会话已过期" }), {
                    status: 401,
                    headers: { "Content-Type": "application/json" },
                }),
            );

        await Promise.allSettled([
            apiRequest({ url: "/api/one" }),
            apiRequest({ url: "/api/two" }),
        ]);

        expect(mocks.messageError).toHaveBeenCalledTimes(1);
        expect(mocks.navigate).toHaveBeenCalledTimes(1);
    });

    it("aborts in-flight requests after a 401 clears the session", async () => {
        useAuthStore.setState({ token: "expired-token", userInfo: mockUserInfo });

        mocks.fetchMock
            .mockImplementationOnce(() =>
                Promise.resolve(
                    new Response(JSON.stringify({ message: "会话已过期" }), {
                        status: 401,
                        headers: { "Content-Type": "application/json" },
                    }),
                ),
            )
            .mockImplementationOnce(
                (_url, init) =>
                    new Promise((_, reject) => {
                        init?.signal?.addEventListener("abort", () => {
                            reject(new DOMException("The operation was aborted", "AbortError"));
                        });
                    }),
            );

        await Promise.allSettled([
            apiRequest({ url: "/api/one" }),
            apiRequest({ url: "/api/two" }),
        ]);

        expect(mocks.messageError).toHaveBeenCalledTimes(1);
        expect(console.debug).toHaveBeenCalledWith("Request aborted");
    });

    it("shows backend messages for 403 responses", async () => {
        mocks.fetchMock.mockResolvedValue(
            new Response(JSON.stringify({ message: "禁止访问" }), {
                status: 403,
                headers: { "Content-Type": "application/json" },
            }),
        );

        await expect(apiRequest({ url: "/api/forbidden" })).rejects.toBeInstanceOf(Response);

        expect(mocks.messageError).toHaveBeenCalledWith("禁止访问");
        expect(mocks.navigate).not.toHaveBeenCalled();
    });

    it("falls back to the default 403 message when the body is not json", async () => {
        mocks.fetchMock.mockResolvedValue(
            new Response("forbidden", {
                status: 403,
                statusText: "Forbidden",
            }),
        );

        await expect(apiRequest({ url: "/api/forbidden-no-json" })).rejects.toBeInstanceOf(
            Response,
        );

        expect(mocks.messageError).toHaveBeenCalledWith("您没有权限执行此操作");
    });

    it("shows a network error message when fetch rejects", async () => {
        mocks.fetchMock.mockRejectedValue(new TypeError("Failed to fetch"));

        await expect(apiRequest({ url: "/api/network" })).rejects.toBeInstanceOf(TypeError);

        expect(mocks.messageError).toHaveBeenCalledWith("网络连接失败，请检查网络连接");
    });

    it("shows business error messages when response code is non-zero", async () => {
        mocks.fetchMock.mockResolvedValue(
            new Response(JSON.stringify({ code: 1001, message: "业务失败", data: null }), {
                status: 200,
                headers: { "Content-Type": "application/json" },
            }),
        );

        await expect(apiRequest({ url: "/api/business" })).rejects.toMatchObject({
            code: 1001,
            message: "业务失败",
        });

        expect(mocks.messageError).toHaveBeenCalledWith("业务失败");
    });

    it("does not show a toast for silent business errors", async () => {
        mocks.fetchMock.mockResolvedValue(
            new Response(JSON.stringify({ code: 1002, message: "静默失败", data: null }), {
                status: 200,
                headers: { "Content-Type": "application/json" },
            }),
        );

        await expect(apiRequest({ url: "/api/silent", silent: true })).rejects.toMatchObject({
            code: 1002,
        });

        expect(mocks.messageError).not.toHaveBeenCalled();
    });

    it("shows a server error message for 500 responses", async () => {
        mocks.fetchMock.mockResolvedValue(
            new Response("boom", {
                status: 500,
                statusText: "Internal Server Error",
            }),
        );

        await expect(apiRequest({ url: "/api/server-error" })).rejects.toBeInstanceOf(Error);

        expect(mocks.messageError).toHaveBeenCalledWith("服务器内部错误，请稍后重试或联系管理员");
    });

    it("treats AbortError as a silent cancellation", async () => {
        mocks.fetchMock.mockRejectedValue(
            new DOMException("The operation was aborted", "AbortError"),
        );

        await expect(apiRequest({ url: "/api/abort" })).rejects.toBeInstanceOf(DOMException);

        expect(mocks.messageError).not.toHaveBeenCalled();
        expect(console.debug).toHaveBeenCalledWith("Request aborted");
    });

    it("downloads files using the response filename", async () => {
        const createObjectURL = vi.fn(() => "blob:download-url");
        const revokeObjectURL = vi.fn();
        const click = vi.fn();
        vi.stubGlobal("URL", {
            createObjectURL,
            revokeObjectURL,
        });
        vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(click);

        mocks.fetchMock.mockResolvedValue(
            new Response(new Blob(["test"], { type: "text/plain" }), {
                status: 200,
                headers: {
                    "content-disposition": "attachment; filename=report.txt",
                },
            }),
        );

        const filename = await apiDownload({ url: "/api/download" });

        expect(filename).toBe("report.txt");
        expect(createObjectURL).toHaveBeenCalledTimes(1);
        expect(click).toHaveBeenCalledTimes(1);
        expect(revokeObjectURL).toHaveBeenCalledWith("blob:download-url");
    });

    it("shows mutation success toasts unless skipSuccessMsg is enabled", async () => {
        mocks.fetchMock
            .mockResolvedValueOnce(
                new Response(JSON.stringify({ code: 0, message: "保存成功", data: { id: 1 } }), {
                    status: 200,
                    headers: { "Content-Type": "application/json" },
                }),
            )
            .mockResolvedValueOnce(
                new Response(JSON.stringify({ code: 0, message: "跳过提示", data: { id: 2 } }), {
                    status: 200,
                    headers: { "Content-Type": "application/json" },
                }),
            );

        await expect(
            apiRequest({ url: "/api/menu", method: "POST", params: { name: "菜单" } }),
        ).resolves.toEqual({ id: 1 });
        await expect(
            apiRequest({
                url: "/api/menu",
                method: "PUT",
                params: { name: "菜单" },
                skipSuccessMsg: true,
            }),
        ).resolves.toEqual({ id: 2 });

        expect(mocks.messageSuccess).toHaveBeenCalledTimes(1);
        expect(mocks.messageSuccess).toHaveBeenCalledWith("保存成功");
    });

    it("shows fallback text for non-json 4xx responses", async () => {
        mocks.fetchMock.mockResolvedValue(
            new Response("bad request", {
                status: 400,
                statusText: "Bad Request",
            }),
        );

        await expect(apiRequest({ url: "/api/bad-request" })).rejects.toBeInstanceOf(Response);

        expect(mocks.messageError).toHaveBeenCalledWith("请求失败：Bad Request");
    });

    it("shows json message text for other 4xx responses", async () => {
        mocks.fetchMock.mockResolvedValue(
            new Response(JSON.stringify({ message: "参数校验失败" }), {
                status: 422,
                statusText: "Unprocessable Entity",
                headers: { "Content-Type": "application/json" },
            }),
        );

        await expect(apiRequest({ url: "/api/validation" })).rejects.toBeInstanceOf(Response);

        expect(mocks.messageError).toHaveBeenCalledWith("参数校验失败");
    });

    it("uses the provided default filename when response headers do not contain one", async () => {
        const createObjectURL = vi.fn(() => "blob:download-url");
        const revokeObjectURL = vi.fn();
        const click = vi.fn();
        vi.stubGlobal("URL", {
            createObjectURL,
            revokeObjectURL,
        });
        vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(click);

        mocks.fetchMock.mockResolvedValue(
            new Response(new Blob(["csv"], { type: "text/csv" }), {
                status: 200,
            }),
        );

        const filename = await apiDownload({ url: "/api/export", filename: "manual.csv" });

        expect(filename).toBe("manual.csv");
        expect(click).toHaveBeenCalledTimes(1);
    });

    it("builds query strings for get requests and preserves explicit post bodies", async () => {
        useAuthStore.setState({ token: "token-123", userInfo: mockUserInfo });
        mocks.fetchMock
            .mockResolvedValueOnce(
                new Response(JSON.stringify({ code: 0, data: { ok: true } }), {
                    status: 200,
                    headers: { "Content-Type": "application/json" },
                }),
            )
            .mockResolvedValueOnce(
                new Response(JSON.stringify({ code: 0, data: { ok: true } }), {
                    status: 200,
                    headers: { "Content-Type": "application/json" },
                }),
            );

        await apiRequest({
            url: "/api/query",
            params: { keyword: "alice", page: "2" },
        });
        await apiRequest({
            url: "/api/body",
            method: "POST",
            body: '{"custom":true}',
            params: { ignored: true },
        });

        expect(mocks.fetchMock.mock.calls[0]?.[0]).toBe("/api/query?keyword=alice&page=2");
        expect(mocks.fetchMock.mock.calls[0]?.[1]).not.toHaveProperty("body");
        expect(new Headers(mocks.fetchMock.mock.calls[0]?.[1]?.headers).get("Authorization")).toBe(
            "Bearer token-123",
        );
        expect(new Headers(mocks.fetchMock.mock.calls[0]?.[1]?.headers).get("Content-Type")).toBe(
            "application/json",
        );
        expect(mocks.fetchMock.mock.calls[1]?.[0]).toBe("/api/body");
        expect(mocks.fetchMock.mock.calls[1]?.[1]).toEqual(
            expect.objectContaining({
                body: '{"custom":true}',
                method: "POST",
            }),
        );
    });

    it("maps pro table responses and defaults total to zero", async () => {
        mocks.fetchMock.mockResolvedValueOnce(
            new Response(JSON.stringify({ code: 0, data: [{ id: 1 }] }), {
                status: 200,
                headers: { "Content-Type": "application/json" },
            }),
        );

        await expect(proTableRequest({ url: "/api/table" })).resolves.toEqual({
            data: [{ id: 1 }],
            total: 0,
            success: true,
        });
    });

    it("handles non-abort DOMException with a cancellation message", async () => {
        mocks.fetchMock.mockRejectedValue(new DOMException("Cancelled", "NetworkError"));

        await expect(apiRequest({ url: "/api/cancel" })).rejects.toBeInstanceOf(DOMException);

        expect(mocks.messageError).toHaveBeenCalledWith("请求被取消");
        expect(console.warn).toHaveBeenCalled();
    });

    it("keeps silent network errors free of toast noise", async () => {
        mocks.fetchMock.mockRejectedValue(new TypeError("offline"));

        await expect(apiRequest({ url: "/api/network", silent: true })).rejects.toBeInstanceOf(
            TypeError,
        );

        expect(mocks.messageError).not.toHaveBeenCalled();
    });

    it("falls back to a mime-derived filename when neither header nor default is provided", async () => {
        const createObjectURL = vi.fn(() => "blob:mime-fallback");
        const revokeObjectURL = vi.fn();
        const click = vi.fn();
        vi.stubGlobal("URL", {
            createObjectURL,
            revokeObjectURL,
        });
        vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(click);
        vi.spyOn(Date, "now").mockReturnValue(1234567890);

        mocks.fetchMock.mockResolvedValue(
            new Response(new Blob(["pdf"], { type: "application/pdf" }), {
                status: 200,
                headers: { "Content-Type": "application/pdf" },
            }),
        );

        const filename = await apiDownload({ url: "/api/file" });

        expect(filename).toBe("1234567890.pdf");
        expect(revokeObjectURL).toHaveBeenCalledWith("blob:mime-fallback");
    });
});
