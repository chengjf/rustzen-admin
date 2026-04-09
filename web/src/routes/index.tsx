import { createFileRoute } from "@tanstack/react-router";

import { DashboardPage } from "@/components/dashboard/DashboardPage";

export const Route = createFileRoute("/")({
    component: DashboardPage,
    notFoundComponent: () => (
        <div className="flex h-full items-center justify-center text-xl font-bold">
            404 Not Found
        </div>
    ),
});
