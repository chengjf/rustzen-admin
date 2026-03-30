import { App } from "antd";
import type { MessageInstance } from "antd/es/message/interface";
import type { ModalStaticFunctions } from "antd/es/modal/confirm";
import type { NotificationInstance } from "antd/es/notification/interface";

import { useAuthStore } from "@/stores/useAuthStore";

let appMessage: MessageInstance;
let appNotify: NotificationInstance;
let appModal: Omit<ModalStaticFunctions, "warn">;

export const MessageContent = () => {
    const staticFunction = App.useApp();
    appMessage = staticFunction.message;
    appModal = staticFunction.modal;
    appNotify = staticFunction.notification;
    return null;
};

export { appMessage, appModal, appNotify };

/**
 * API request adapter
 */
export const apiRequest = <T, P = Api.BaseParams>(props: RequestOptions<P>) => {
    return coreRequest<T, P>(props).then((res) => res.data);
};

/**
 * Download file
 */
export const apiDownload = async <T>({
    filename,
    ...options
}: RequestOptions<T> & { filename?: string }): Promise<string> => {
    const { url, config, reqDelete } = formatFetchConfig(options);
    try {
        const response = await fetch(url, config);
        if (!response.ok) {
            throw response;
        }
        return downloadFile(response, filename);
    } catch (error) {
        return handleError(error, options); // Pass options to handleError
    } finally {
        reqDelete();
    }
};
/**
 * ProTable request adapter
 */
export const proTableRequest = <T, P = Api.BaseParams>(
    props: RequestOptions<P>,
): Promise<Api.PageResponse<T>> => {
    return coreRequest<T, P>(props).then((res) => ({
        data: res.data as T[],
        total: res.total || 0,
        success: true,
    }));
};

const requestPool = new Set<AbortController>();

/**
 * Get auth headers from localStorage
 */
export const getAuthHeaders = (): Record<string, string> => {
    const token = useAuthStore.getState().token;
    return token ? { Authorization: `Bearer ${token}` } : {};
};

/**
 * Default request headers
 */
const defaultHeaders = {
    "Content-Type": "application/json",
};

interface RequestOptions<P = Api.BaseParams> extends RequestInit {
    /**
     * Request url
     */
    url: string;
    /**
     * Request params
     */
    params?: P;
    /**
     * Custom success message
     */
    successMessage?: string;

    /**
     * Custom error message
     */
    errorMessage?: string;

    /**
     * If true, disables all messages
     */
    silent?: boolean;

    /**
     * If true, skip success message
     */
    skipSuccessMsg?: boolean;
}

/**
 * Core request function with unified error and success handling
 */
const coreRequest = async <T, P>(options: RequestOptions<P>): Promise<Api.ApiResponse<T>> => {
    const { url, config, reqDelete } = formatFetchConfig(options);
    try {
        const response = await fetch(url, config);
        if (!response.ok) {
            throw response;
        }
        const result = await response.json();
        if (result.code !== 0) {
            return Promise.reject(result);
        }
        const isMutation = ["post", "put", "delete", "patch"].includes(
            config.method?.toLowerCase() || "",
        );

        // 允许单个接口通过配置"跳过"自动提示
        // 在调用时传：menuAPI.create(data, { skipSuccessMsg: true })
        const skipMsg = options.skipSuccessMsg;

        if (isMutation && !skipMsg) {
            appMessage.success(result.message || "操作成功");
        }
        return result;
    } catch (error) {
        return handleError(error, options); // Pass options to handleError
    } finally {
        reqDelete();
    }
};

const formatFetchConfig = <T>({ params, url, ...options }: RequestOptions<T>) => {
    const controller = new AbortController();
    requestPool.add(controller);

    const config: RequestInit = {
        ...options,
        signal: controller.signal,
        headers: {
            ...defaultHeaders,
            ...Object.fromEntries(new Headers(options.headers || {}).entries()),
            ...getAuthHeaders(),
        },
    };
    if (["PUT", "POST"].includes(options.method || "GET")) {
        config.body = options.body || JSON.stringify(params);
    } else {
        url += buildQueryString(params);
    }
    return {
        url,
        config,
        reqDelete: () => requestPool.delete(controller),
    };
};

/**
 * Handle all errors
 */
const handleError = async (error: unknown, options?: RequestOptions<any>): Promise<never> => {
    // 1. AbortError (request cancellation or timeout)
    if (error instanceof DOMException) {
        if (error.name === "AbortError") {
            console.debug("Request aborted");
            // Optionally show timeout message if needed
            // if (!options?.silent) {
            //     appMessage.error("请求超时，请重试");
            // }
            return Promise.reject(error);
        }
        // Other DOMException types
        if (!options?.silent) {
            appMessage.error("请求被取消");
        }
        console.warn("Request cancelled:", error);
        return Promise.reject(error);
    }

    // 2. HTTP Response errors
    if (error instanceof Response) {
        const response = error;
        const statusCode = response.status;

        if (statusCode === 401) {
            let errorMsg = "会话过期，请重新登录";
            try {
                const errorData = await response.json();
                errorMsg = errorData.message || errorMsg;
            } catch {
                // Cannot parse JSON, use default message
            }
            useAuthStore.getState().clearAuth();
            // Abort all pending requests
            requestPool.forEach((controller) => {
                if (!controller.signal.aborted) {
                    controller.abort();
                }
            });
            requestPool.clear(); // Clean up the pool
            if (!options?.silent) {
                appMessage.error(errorMsg);
            }
            return Promise.reject(error);
        }

        if (statusCode === 403) {
            if (!options?.silent) {
                appMessage.error("您没有权限执行此操作");
            }
            return Promise.reject(error);
        }

        if (statusCode >= 500) {
            if (!options?.silent) {
                appMessage.error("服务器内部错误，请稍后重试或联系管理员");
            }
            return Promise.reject(new Error(response.statusText));
        }

        // Other 4xx client errors
        try {
            const errorData = await response.json();
            const messageText = errorData.message || response.statusText || "请求失败";
            if (!options?.silent) {
                appMessage.error(messageText);
            }
        } catch {
            // Cannot parse JSON body
            if (!options?.silent) {
                appMessage.error(`请求失败：${response.statusText}`);
            }
        }
        return Promise.reject(error);
    }

    // 3. Network errors (TypeError for CORS/offline/DNS, etc.)
    if (!options?.silent) {
        appMessage.error("网络连接失败，请检查网络连接");
    }
    console.error("[Network Error]:", error);
    return Promise.reject(error);
};

/**
 * Safe params conversion
 */
const buildQueryString = <P>(params?: P): string => {
    if (!params) return "";
    const searchParams = new URLSearchParams(params);
    const query = searchParams.toString();
    return query ? `?${query}` : "";
};

/**
 * Download file
 */
const downloadFile = async (response: Response, defaultName?: string): Promise<string> => {
    const blob = await response.blob();
    const contentDisposition = response.headers.get("content-disposition");
    const filename = contentDisposition?.split("filename=")[1] || defaultName;
    const downloadName = filename || `${Date.now()}${getFileExt(blob.type)}`;
    return new Promise((resolve) => {
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = downloadName;
        document.body.appendChild(a);
        a.click();
        URL.revokeObjectURL(url);
        document.body.removeChild(a);
        resolve(downloadName);
    });
};

// 根据 blob 类型确定文件扩展名
const getFileExt = (mimeType: string): string => {
    const mimeToExt: Record<string, string> = {
        // 文本文件
        "text/plain": ".txt",
        "text/csv": ".csv",
        "text/html": ".html",
        "text/css": ".css",
        "text/javascript": ".js",
        "text/xml": ".xml",
        "text/markdown": ".md",

        // 文档文件
        "application/pdf": ".pdf",
        "application/msword": ".doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document": ".docx",
        "application/vnd.ms-excel": ".xls",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet": ".xlsx",
        "application/vnd.ms-powerpoint": ".ppt",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation": ".pptx",

        // 压缩文件
        "application/zip": ".zip",
        "application/x-rar-compressed": ".rar",
        "application/x-7z-compressed": ".7z",
        "application/gzip": ".gz",
        "application/x-tar": ".tar",

        // 图片文件
        "image/jpeg": ".jpg",
        "image/png": ".png",
        "image/gif": ".gif",
        "image/svg+xml": ".svg",
        "image/webp": ".webp",
        "image/bmp": ".bmp",
        "image/tiff": ".tiff",

        // 音频文件
        "audio/mpeg": ".mp3",
        "audio/wav": ".wav",
        "audio/ogg": ".ogg",
        "audio/mp4": ".m4a",

        // 视频文件
        "video/mp4": ".mp4",
        "video/avi": ".avi",
        "video/quicktime": ".mov",
        "video/x-msvideo": ".avi",
        "video/webm": ".webm",

        // JSON 和 XML
        "application/json": ".json",
        "application/xml": ".xml",

        // 其他常见类型
        "application/octet-stream": ".bin",
        "application/x-binary": ".bin",
    };

    return mimeToExt[mimeType] || ".bin";
};
