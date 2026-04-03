import { LockOutlined, UserOutlined } from "@ant-design/icons";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Button, Card, Form, Input, Typography } from "antd";
import { useState, useCallback } from "react";

import { appMessage } from "@/api";
import { authAPI } from "@/api/auth";
import type { LoginRequest } from "@/api/types/LoginRequest";
import { useAuthStore } from "@/stores/useAuthStore";

// =============================================================================
// 1. 路由定义
// =============================================================================

export const Route = createFileRoute("/login")({
    component: LoginPage,
});

// =============================================================================
// 2. 页面主组件
// =============================================================================

function LoginPage() {
    const navigate = useNavigate();
    const [loading, setLoading] = useState(false);
    const { handleLogin } = useAuthStore();

    /**
     * 登录逻辑处理
     */
    const onLogin = useCallback(
        async (values: LoginRequest) => {
            setLoading(true);
            try {
                const res = await authAPI.login(values);

                // 存储 token 和用户信息
                handleLogin(res.token, res.userInfo);

                appMessage.success("登录成功，正在跳转...");

                // 使用 replace 防止用户通过后退键回到登录页
                void navigate({ to: "/", replace: true });
            } catch (error: any) {
                console.error("[Login Failed]:", error);
            } finally {
                setLoading(false);
            }
        },
        [handleLogin, navigate],
    );

    return (
        <div className="flex min-h-screen items-center justify-center bg-gradient-to-br from-blue-50 via-slate-100 to-blue-100">
            <Card
                bordered={false}
                className="w-full max-w-[400px] shadow-lg border-t-4 border-blue-500"
            >
                <div className="mb-10 text-center">
                    <Typography.Title level={2} style={{ margin: 0, color: "#1890ff" }}>
                        Rustzen Admin
                    </Typography.Title>
                    <Typography.Text type="secondary">
                        基于 Rust & React 的高性能后台管理系统
                    </Typography.Text>
                </div>

                <Form
                    name="login"
                    onFinish={onLogin}
                    autoComplete="off"
                    size="large"
                    layout="vertical"

                >
                    <Form.Item
                        name="username"
                        rules={[
                            { required: true, message: "请输入用户名" },
                            { min: 3, message: "用户名长度不能少于3位" },
                        ]}
                    >
                        <Input
                            prefix={<UserOutlined className="text-gray-300" />}
                            placeholder="用户名"
                            disabled={loading}
                        />
                    </Form.Item>

                    <Form.Item
                        name="password"
                        rules={[
                            { required: true, message: "请输入密码" },
                            { min: 6, message: "密码长度不能少于6位" },
                        ]}
                    >
                        <Input.Password
                            prefix={<LockOutlined className="text-gray-300" />}
                            placeholder="密码"
                            disabled={loading}
                        />
                    </Form.Item>

                    <Form.Item className="mb-0">
                        <Button
                            type="primary"
                            htmlType="submit"
                            loading={loading}
                            className="w-full h-11 text-lg font-medium"
                        >
                            {loading ? "验证中..." : "登 录"}
                        </Button>
                    </Form.Item>
                </Form>

                <div className="mt-6 text-center text-gray-400 text-xs">
                    © 2026 Rustzen Team. All Rights Reserved.
                </div>
            </Card>
        </div>
    );
}
