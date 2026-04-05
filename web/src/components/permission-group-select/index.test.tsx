import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("antd", () => ({
    Checkbox: ({
        children,
        checked,
        onChange,
        disabled,
    }: {
        children?: React.ReactNode;
        checked?: boolean;
        onChange?: (event: { target: { checked: boolean } }) => void;
        disabled?: boolean;
    }) => (
        <label>
            <input
                type="checkbox"
                checked={checked}
                disabled={disabled}
                onChange={(event) => onChange?.({ target: { checked: event.target.checked } })}
            />
            {children}
        </label>
    ),
    Col: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    Divider: () => <hr />,
    Input: {
        Search: ({
            placeholder,
            value,
            onChange,
        }: {
            placeholder?: string;
            value?: string;
            onChange?: (event: { target: { value: string } }) => void;
        }) => (
            <input
                placeholder={placeholder}
                value={value}
                onChange={(event) => onChange?.({ target: { value: event.target.value } })}
            />
        ),
    },
    Row: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
    Tag: ({ children }: { children: React.ReactNode }) => <span>{children}</span>,
    Typography: {
        Text: ({ children }: { children: React.ReactNode }) => <span>{children}</span>,
    },
}));

import { PermissionGroupSelect } from ".";

const options = [
    { value: 1, label: "用户列表", code: "system:user:list" },
    { value: 2, label: "用户创建", code: "system:user:create" },
    { value: 3, label: "角色列表", code: "system:role:list" },
];

describe("PermissionGroupSelect", () => {
    it("groups permissions and filters them by search text", () => {
        render(<PermissionGroupSelect options={options} />);

        expect(screen.getByText("system:role")).toBeInTheDocument();
        expect(screen.getByText("system:user")).toBeInTheDocument();

        fireEvent.change(screen.getByPlaceholderText("搜索权限名称或代码"), {
            target: { value: "创建" },
        });

        expect(screen.getByText("用户创建")).toBeInTheDocument();
        expect(screen.queryByText("用户列表")).not.toBeInTheDocument();
    });

    it("selects an entire group and updates the count", () => {
        const onChange = vi.fn();
        render(<PermissionGroupSelect options={options} onChange={onChange} />);

        fireEvent.click(screen.getAllByRole("checkbox")[1]);

        expect(onChange).toHaveBeenCalledWith([3]);
    });

    it("shows partial selection state through the summary count", () => {
        render(<PermissionGroupSelect options={options} value={[1]} />);

        expect(screen.getByText("已选择 1 项权限")).toBeInTheDocument();
    });
});
