import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useAuthStore } from "@/stores/useAuthStore";
import { mockUserInfo } from "@/test/mocks/handlers";

const mocks = vi.hoisted(() => ({
    changePassword: vi.fn(),
    logout: vi.fn(),
    success: vi.fn(),
    navigate: vi.fn(),
}));

vi.mock("@tanstack/react-router", () => ({
    useNavigate: () => mocks.navigate,
}));

vi.mock("@ant-design/pro-components", () => ({
    ModalForm: ({
        trigger,
        children,
        onFinish,
    }: {
        trigger?: React.ReactNode;
        children?: React.ReactNode;
        onFinish?: (values: any) => Promise<boolean>;
    }) => (
        <div>
            <button
                onClick={() =>
                    onFinish?.({
                        oldPassword: "old-pass",
                        newPassword: "new-pass",
                        confirmPassword: "new-pass",
                    })
                }
            >
                {trigger}
            </button>
            {children}
        </div>
    ),
    ProFormText: Object.assign(() => null, { Password: () => null }),
}));

vi.mock("antd", () => ({
    Form: {
        useForm: () => [{}],
    },
}));

vi.mock("@/api", () => ({
    appMessage: {
        success: mocks.success,
    },
}));

vi.mock("@/api/auth", () => ({
    authAPI: {
        changePassword: mocks.changePassword,
        logout: mocks.logout,
    },
}));

import { ChangePasswordModal } from "./ChangePasswordModal";

beforeEach(() => {
    useAuthStore.setState({
        token: "token",
        userInfo: mockUserInfo,
    });
    mocks.changePassword.mockResolvedValue(undefined);
    mocks.logout.mockResolvedValue(undefined);
});

afterEach(() => {
    useAuthStore.setState({ token: null, userInfo: null });
    vi.clearAllMocks();
});

describe("ChangePasswordModal", () => {
    it("changes password, logs out, clears auth, and redirects", async () => {
        render(<ChangePasswordModal />);

        fireEvent.click(screen.getByRole("button", { name: "修改密码" }));

        await waitFor(() => {
            expect(mocks.changePassword).toHaveBeenCalledWith({
                oldPassword: "old-pass",
                newPassword: "new-pass",
            });
        });

        expect(mocks.logout).toHaveBeenCalledTimes(1);
        expect(useAuthStore.getState().token).toBeNull();
        expect(mocks.success).toHaveBeenCalledWith("密码修改成功，请使用新密码重新登录");
        expect(mocks.navigate).toHaveBeenCalledWith({ to: "/login" });
    });
});
