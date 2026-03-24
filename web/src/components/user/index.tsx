import { ModalForm, ProFormText } from "@ant-design/pro-components";
import { Form } from "antd";

import { useAuthStore } from "@/stores/useAuthStore";

import { UserAvatar } from "./avatar";

export const UserProfileModal = () => {
    const { userInfo } = useAuthStore();
    const [form] = Form.useForm();
    return (
        <ModalForm
            readonly
            form={form}
            title="用户信息"
            trigger={<span>个人信息</span>}
            layout="horizontal"
            labelCol={{ span: 6 }}
            submitter={false}
            onOpenChange={(visible) => {
                if (visible) {
                    form.setFieldsValue(userInfo);
                }
            }}
        >
            <div className="flex pt-5">
                <div className="flex-1">
                    <ProFormText name="username" label="用户名" readonly />
                    <ProFormText name="email" label="邮箱" readonly />
                    <ProFormText name="phone" label="手机号" />
                    <ProFormText name="realName" label="真实姓名" />
                </div>
                <div className="flex w-70 flex-none flex-col items-center p-10">
                    <UserAvatar />
                </div>
            </div>
        </ModalForm>
    );
};
