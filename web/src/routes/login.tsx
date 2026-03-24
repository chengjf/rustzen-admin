import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute } from "@tanstack/react-router";
import { useNavigate } from "@tanstack/react-router";
import { Button, Card, Form, Input, Typography } from "antd";
import { useTransition } from "react";

import { authAPI } from "@/api/auth";
import { useAuthStore } from "@/stores/useAuthStore";
export const Route = createFileRoute("/login")({
    component: () => <LoginPage />,
});

function LoginPage() {
    const navigate = useNavigate();
    const [isPending, startTransition] = useTransition();
    const { handleLogin } = useAuthStore();
    const onLogin = async (values: Auth.LoginRequest) => {
        startTransition(async () => {
            try {
                const res = await authAPI.login(values);
                handleLogin(res.token, res.userInfo);
                navigate({ to: "/", replace: true });
            } catch (error) {
                console.error("Login failed", error);
            }
        });
    };

    return (
        <div className="flex min-h-screen items-center justify-center bg-gray-50">
            <Card className="w-96">
                <div className="mb-8 text-center">
                    <Typography.Title level={2} className="mb-2">
                        Rustzen Admin
                    </Typography.Title>
                </div>
                <Form
                    name="login"
                    onFinish={onLogin}
                    autoComplete="off"
                    size="large"
                    initialValues={{
                        username: "superadmin",
                        password: "rustzen@123",
                    }}
                >
                    <Form.Item
                        name="username"
                        rules={[
                            {
                                required: true,
                                message: "请输入用户名",
                            },
                            {
                                min: 3,
                                message:
                                    "用户名至少包含3个字符",
                            },
                        ]}
                    >
                        <Input
                            prefix={<UserOutlined />}
                            placeholder="用户名"
                        />
                    </Form.Item>
                    <Form.Item
                        name="password"
                        rules={[
                            {
                                required: true,
                                message: "请输入密码",
                            },
                            {
                                min: 6,
                                message:
                                    "密码至少包含6个字符",
                            },
                        ]}
                    >
                        <Input.Password
                            prefix={<LockOutlined />}
                            placeholder="密码"
                        />
                    </Form.Item>
                    <Form.Item>
                        <Button
                            type="primary"
                            htmlType="submit"
                            loading={isPending}
                            className="w-full"
                        >
                            登录
                        </Button>
                    </Form.Item>
                </Form>
            </Card>
        </div>
    );
}
