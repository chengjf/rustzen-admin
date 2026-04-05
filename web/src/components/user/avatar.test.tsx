import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { mockUserInfo } from "@/test/mocks/handlers";
import { useAuthStore } from "@/stores/useAuthStore";

const mocks = vi.hoisted(() => ({
    error: vi.fn(),
}));

vi.mock("antd", () => ({
    Avatar: ({ src }: { src?: string | null }) => <div>{src || "avatar"}</div>,
    Upload: ({
        children,
        onChange,
    }: {
        children?: React.ReactNode;
        onChange?: (info: { file: { status: string; response: { data: string } } }) => void;
    }) => (
        <div>
            <button
                onClick={() =>
                    onChange?.({
                        file: { status: "done", response: { data: "https://cdn/avatar.png" } },
                    })
                }
            >
                upload-trigger
            </button>
            {children}
        </div>
    ),
}));

vi.mock("@/api", () => ({
    appMessage: {
        error: mocks.error,
    },
}));

import { UserAvatar, beforeUpload } from "./avatar";

beforeEach(() => {
    act(() => {
        useAuthStore.setState({
            token: "token",
            userInfo: { ...mockUserInfo, avatarUrl: null },
        });
    });
});

afterEach(() => {
    act(() => {
        useAuthStore.setState({ token: null, userInfo: null });
    });
    vi.clearAllMocks();
});

describe("UserAvatar", () => {
    it("updates the avatar when upload succeeds", () => {
        render(<UserAvatar />);

        act(() => {
            fireEvent.click(screen.getByRole("button", { name: "upload-trigger" }));
        });

        expect(useAuthStore.getState().userInfo?.avatarUrl).toBe("https://cdn/avatar.png");
    });

    it("rejects unsupported file types", async () => {
        const result = await beforeUpload({
            type: "image/gif",
            size: 100,
        } as any);

        expect(result).toBe(false);
        expect(mocks.error).toHaveBeenCalledWith("You can only upload JPG/JPEG/PNG file!");
    });

    it("rejects files larger than 1MB", async () => {
        const result = await beforeUpload({
            type: "image/png",
            size: 2 * 1024 * 1024,
        } as any);

        expect(result).toBe(false);
        expect(mocks.error).toHaveBeenCalledWith("Image must smaller than 1MB!");
    });
});
