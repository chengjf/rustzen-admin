import { createFileRoute, Link } from "@tanstack/react-router"; // 导入 Link 用于跳转

export const Route = createFileRoute("/403")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        // 使用 flexbox 将内容垂直和水平居中，占满全屏高度
        <div className="flex flex-col items-center justify-center min-h-screen bg-gray-50 text-gray-900 px-4">
            <div className="text-center">
                {/* 醒目的 403 状态码 */}
                <h1 className="text-6xl font-extrabold text-red-600 sm:text-7xl">
                    403
                </h1>
                
                {/* 主要错误信息 */}
                <p className="mt-4 text-2xl font-bold tracking-tight sm:text-3xl text-gray-900">
                    访问被拒绝
                </p>
                
                {/* 详细描述信息 */}
                <p className="mt-6 text-base leading-7 text-gray-600 max-w-md mx-auto">
                    抱歉，您没有权限访问该资源或执行此操作。如果您认为这是一个错误，请联系系统管理员。
                </p>
                
                {/* 操作按钮区域 */}
                <div className="mt-10 flex items-center justify-center gap-x-6">
                    <Link
                        to="/" // 跳转到首页
                        className="rounded-md bg-red-600 px-5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-red-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-600 transition-colors duration-200"
                    >
                        返回首页
                    </Link>
                    {/* 可选：添加一个“联系支持”的链接 */}
                    {/* <a href="#" className="text-sm font-semibold text-gray-900 hover:text-red-600">
                        联系支持 <span aria-hidden="true">&rarr;</span>
                    </a> */}
                </div>
            </div>
        </div>
    );
}