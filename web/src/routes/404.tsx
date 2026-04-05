import { createFileRoute, Link } from "@tanstack/react-router";

export const Route = createFileRoute("/404")({
    component: RouteComponent,
});

export function RouteComponent() {
    return (
        // 保持与 403 一致的 Flex 居中布局
        <div className="flex flex-col items-center justify-center min-h-screen bg-gray-50 text-gray-900 px-4">
            <div className="text-center">
                {/* 状态码：改用蓝色系，区分权限错误的红色 */}
                <h1 className="text-6xl font-extrabold text-blue-600 sm:text-7xl">404</h1>

                {/* 主要标题 */}
                <p className="mt-4 text-2xl font-bold tracking-tight text-gray-900 sm:text-3xl">
                    页面走丢了
                </p>

                {/* 描述信息 */}
                <p className="mt-6 text-base leading-7 text-gray-600 max-w-md mx-auto">
                    抱歉，我们找不到您要访问的页面。它可能已被移动、删除，或者您输入的地址有误。
                </p>

                {/* 操作区域 */}
                <div className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4">
                    <Link
                        to="/"
                        className="w-full sm:w-auto rounded-md bg-blue-600 px-6 py-3 text-sm font-semibold text-white shadow-sm hover:bg-blue-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 transition-all duration-200"
                    >
                        返回首页
                    </Link>

                    {/* 增加一个辅助按钮：返回上一页 */}
                    <button
                        onClick={() => window.history.back()}
                        className="w-full sm:w-auto rounded-md bg-white px-6 py-3 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 transition-all duration-200"
                    >
                        返回上一页
                    </button>
                </div>
            </div>

            {/* 可选：底部微弱提示 */}
            <div className="absolute bottom-8 text-sm text-gray-400">
                &copy; {new Date().getFullYear()} Your System Name. All rights reserved.
            </div>
        </div>
    );
}
