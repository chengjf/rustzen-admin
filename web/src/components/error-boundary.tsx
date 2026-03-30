import { Button, Result } from "antd";
import React from "react";

interface ErrorBoundaryProps {
    children: React.ReactNode;
}

interface ErrorBoundaryState {
    hasError: boolean;
    error?: Error;
}

/**
 * Error Boundary Component
 * Catches JavaScript errors anywhere in the child component tree and displays a fallback UI.
 */
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
    constructor(props: ErrorBoundaryProps) {
        super(props);
        this.state = { hasError: false };
    }

    static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        // Log error to console in development
        console.error("[Unhandled Component Error]", error, errorInfo);
        // TODO: Send to error monitoring service (Sentry, LogRocket, etc.)
        // if (import.meta.env.PROD) {
        //     sendToSentry({ error, errorInfo, url: window.location.href });
        // }
    }

    handleReset = () => {
        this.setState({ hasError: false, error: undefined });
    };

    render() {
        if (this.state.hasError) {
            return (
                <Result
                    status="error"
                    title="出错了"
                    subTitle="抱歉，页面遇到了一个错误。请尝试刷新页面。"
                    extra={
                        <Button type="primary" onClick={this.handleReset}>
                            重试
                        </Button>
                    }
                />
            );
        }

        return this.props.children;
    }
}
