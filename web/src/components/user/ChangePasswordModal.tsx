import { ModalForm, ProFormText } from "@ant-design/pro-components";
import { Form } from "antd";
import { useNavigate } from "@tanstack/react-router";

import { appMessage } from "@/api";
import { authAPI } from "@/api/auth";
import { useAuthStore } from "@/stores/useAuthStore";

export const ChangePasswordModal = () => {
    const [form] = Form.useForm();
    const navigate = useNavigate();

    return (
        <ModalForm
            form={form}
            title="修改密码"
            trigger={
                <span>
                    修改密码
                </span>
            }
            layout="horizontal"
            labelCol={{ span: 6 }}
            submitter={{
                searchConfig: {
                    submitText: "确认修改",
                    resetText: "取消",
                },
            }}
            onFinish={async (values) => {
                await authAPI.changePassword({
                    oldPassword: values.oldPassword,
                    newPassword: values.newPassword,
                });
                await authAPI.logout();
                useAuthStore.getState().clearAuth();
                appMessage.success("密码修改成功，请使用新密码重新登录");
                void navigate({ to: "/login" });
                return true;
            }}
        >
            <ProFormText.Password
                name="oldPassword"
                label="旧密码"
                rules={[{ required: true, message: "请输入旧密码" }]}
            />
            <ProFormText.Password
                name="newPassword"
                label="新密码"
                rules={[
                    { required: true, message: "请输入新密码" },
                    { min: 6, message: "至少6个字符" },
                ]}
            />
            <ProFormText.Password
                name="confirmPassword"
                label="确认密码"
                dependencies={["newPassword"]}
                rules={[
                    { required: true, message: "请确认密码" },
                    ({ getFieldValue }) => ({
                        validator(_, value) {
                            if (!value || getFieldValue("newPassword") === value) {
                                return Promise.resolve();
                            }
                            return Promise.reject(new Error("两次密码不一致"));
                        },
                    }),
                ]}
            />
        </ModalForm>
    );
};
