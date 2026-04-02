import { createRouter } from "@tanstack/react-router";

import * as TanStackQueryProvider from "./integrations/tanstack-query/root-provider.tsx";
import { routeTree } from "./routeTree.gen";

export const TanStackQueryProviderContext = TanStackQueryProvider.getContext();

export const router = createRouter({
    routeTree,
    context: {
        ...TanStackQueryProviderContext,
    },
    defaultPreload: "intent",
    scrollRestoration: true,
    defaultStructuralSharing: true,
    defaultPreloadStaleTime: 0,
});

declare module "@tanstack/react-router" {
    interface Register {
        router: typeof router;
    }
    interface StaticDataRouteOption {
        headerTitle?: string;
        headerHide?: boolean;
        headerBack?: boolean;
    }
}
