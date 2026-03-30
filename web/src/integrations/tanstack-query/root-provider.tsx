import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

// eslint-disable-next-line react-refresh/only-export-components
export function getContext() {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                refetchInterval: 1000 * 60 * 10,
                refetchIntervalInBackground: true,
                staleTime: 0,
                gcTime: 1000 * 60 * 30,
                refetchOnWindowFocus: true,
                refetchOnReconnect: true,
                retry: 0, // Retry failed queries once
            },
            mutations: {
                retry: false, // Don't retry mutations (user sees error)
            },
        },
    });

    return {
        queryClient,
    };
}

export function Provider({
    children,
    queryClient,
}: {
    children: React.ReactNode;
    queryClient: QueryClient;
}) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}
