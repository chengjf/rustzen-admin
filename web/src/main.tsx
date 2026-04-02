import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import ReactDOM from "react-dom/client";

import { ErrorBoundary } from "./components/error-boundary";
import * as TanStackQueryProvider from "./integrations/tanstack-query/root-provider.tsx";
import { router, TanStackQueryProviderContext } from "./router";

// Render the root
const rootElement = document.getElementById("root");
if (rootElement && !rootElement.innerHTML) {
    const root = ReactDOM.createRoot(rootElement);
    root.render(
        <StrictMode>
            <ErrorBoundary>
                <TanStackQueryProvider.Provider {...TanStackQueryProviderContext}>
                    <RouterProvider router={router} />
                </TanStackQueryProvider.Provider>
            </ErrorBoundary>
        </StrictMode>,
    );
}
