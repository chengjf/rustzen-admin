import { Popconfirm } from "antd";
import type { ButtonProps } from "antd/lib/button";
import React from "react";

import { appModal } from "@/api";
import { useAuthStore } from "@/stores/useAuthStore";

interface AuthWrapProps {
    code: string;
    children: React.ReactNode;
    hidden?: boolean;
    fallback?: React.ReactNode;
}

export const AuthWrap: React.FC<AuthWrapProps> = ({
    code,
    children,
    hidden = false,
    fallback = null,
}) => {
    const isPermission = useAuthStore.getState().checkPermissions(code);
    if (isPermission && !hidden) {
        return children;
    }
    return fallback;
};

interface AuthPopconfirmProps extends AuthWrapProps {
    title: React.ReactNode;
    description?: React.ReactNode;
    onConfirm: () => Promise<void>;
    onCancel?: () => Promise<void>;
    okButtonProps?: ButtonProps;
    cancelButtonProps?: ButtonProps;
}

export const AuthPopconfirm: React.FC<AuthPopconfirmProps> = ({
    code,
    children,
    hidden = false,
    title,
    description,
    onConfirm,
    onCancel,
}) => {
    return (
        <AuthWrap code={code} hidden={hidden}>
            <Popconfirm
                placement="leftBottom"
                title={title}
                description={description}
                onConfirm={onConfirm}
                onCancel={onCancel}
                okText="确认"
                cancelText="取消"
            >
                {children}
            </Popconfirm>
        </AuthWrap>
    );
};

interface AuthConfirmProps extends AuthPopconfirmProps {
    className?: string;
}

export const AuthConfirm: React.FC<AuthConfirmProps> = (props) => {
    const handleConfirm = () => {
        appModal.confirm({
            title: props.title,
            content: props.description,
            onOk: props.onConfirm,
            onCancel: props.onCancel,
            okText: "确认",
            cancelText: "取消",
            okButtonProps: props.okButtonProps,
            cancelButtonProps: props.cancelButtonProps,
        });
    };

    return (
        <AuthWrap code={props.code} hidden={props.hidden}>
            <span onClick={handleConfirm} className={props.className}>
                {props.children}
            </span>
        </AuthWrap>
    );
};
