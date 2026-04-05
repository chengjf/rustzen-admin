import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const mocks = vi.hoisted(() => ({
    setFieldsValue: vi.fn(),
}));

vi.mock("@ant-design/pro-components", () => ({
    ModalForm: ({
        trigger,
        children,
        onOpenChange,
    }: {
        trigger?: React.ReactNode;
        children?: React.ReactNode;
        onOpenChange?: (visible: boolean) => void;
    }) => (
        <div>
            <button onClick={() => onOpenChange?.(true)}>{trigger}</button>
            {children}
        </div>
    ),
    ProFormText: ({ label }: { label?: React.ReactNode }) => <div>{label}</div>,
}));

vi.mock("antd", () => ({
    Form: {
        useForm: () => [
            {
                setFieldsValue: mocks.setFieldsValue,
            },
        ],
    },
}));

vi.mock("./avatar", () => ({
    UserAvatar: () => <div>user-avatar</div>,
}));

import { UserProfileModal } from "./index";

beforeEach(() => {
    act(() => {
        useAuthStore.setState({
            token: "token",
            userInfo: mockUserInfo,
        });
    });
});

afterEach(() => {
    act(() => {
        useAuthStore.setState({ token: null, userInfo: null });
    });
    vi.clearAllMocks();
});

describe("UserProfileModal", () => {
    it("loads user info into the form when opened", () => {
        render(<UserProfileModal />);

        act(() => {
            fireEvent.click(screen.getByRole("button", { name: "个人信息" }));
        });

        expect(mocks.setFieldsValue).toHaveBeenCalledWith(mockUserInfo);
        expect(screen.getByText("user-avatar")).toBeInTheDocument();
        expect(screen.getByText("用户名")).toBeInTheDocument();
    });
});
